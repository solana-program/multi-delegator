use crate::{
    helpers::{
        transfer_with_delegate, validate_recurring_transfer, Delegation, TransferAccounts,
        TransferData,
    },
    state::RecurringDelegation,
    AccountCheck, MultiDelegateAccount, MultiDelegatorError, ProgramAccount, SignerAccount,
    TokenAccountInterface, TokenProgramInterface,
};
use pinocchio::{
    error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    AccountView, ProgramResult,
};

pub const DISCRIMINATOR: &u8 = &5;

pub fn process(accounts: &[AccountView], transfer_data: &TransferData) -> ProgramResult {
    let accounts_struct = RecurringTransferAccounts::try_from(accounts)?;

    let current_ts = Clock::get()?.unix_timestamp;
    {
        let mut binding = accounts_struct.delegation_pda.try_borrow_mut()?;
        let delegation_mut = RecurringDelegation::load_mut(&mut binding)?;

        // Fail fast: Check authorization first
        Delegation::check(
            &delegation_mut.header,
            &transfer_data.delegator,
            accounts_struct.delegatee.address(),
        )?;

        let mut period_start = delegation_mut.current_period_start_ts;
        let mut pulled = delegation_mut.amount_pulled_in_period;
        validate_recurring_transfer(
            transfer_data.amount,
            delegation_mut.amount_per_period,
            delegation_mut.period_length_s,
            &mut period_start,
            &mut pulled,
            delegation_mut.expiry_ts,
            current_ts,
        )?;
        delegation_mut.current_period_start_ts = period_start;
        delegation_mut.amount_pulled_in_period = pulled;
    }

    transfer_with_delegate(
        transfer_data.amount,
        &transfer_data.delegator,
        &transfer_data.mint,
        &TransferAccounts {
            delegator_ata: accounts_struct.delegator_ata,
            to_ata: accounts_struct.receiver_ata,
            multidelegate_pda: accounts_struct.multi_delegate,
            token_program: accounts_struct.token_program,
        },
    )?;

    Ok(())
}

pub struct RecurringTransferAccounts<'a> {
    pub delegation_pda: &'a AccountView,
    pub multi_delegate: &'a AccountView,
    pub delegator_ata: &'a AccountView,
    pub receiver_ata: &'a AccountView,
    pub token_program: &'a AccountView,
    pub delegatee: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for RecurringTransferAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [delegation_pda, multi_delegate, delegator_ata, receiver_ata, token_program, delegatee] =
            accounts
        else {
            return Err(MultiDelegatorError::NotEnoughAccountKeys.into());
        };

        ProgramAccount::check(delegation_pda)?;
        MultiDelegateAccount::check(multi_delegate)?;
        TokenProgramInterface::check(token_program)?;
        TokenAccountInterface::check_accounts_with_program(
            token_program,
            &[delegator_ata, receiver_ata],
        )?;
        SignerAccount::check(delegatee)?;

        Ok(Self {
            delegation_pda,
            multi_delegate,
            delegator_ata,
            receiver_ata,
            token_program,
            delegatee,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::utils::build_and_send_transaction;
    use crate::{
        state::RecurringDelegation,
        tests::{
            asserts::TransactionResultExt,
            constants::{MINT_DECIMALS, TOKEN_PROGRAM_ID},
            utils::{
                current_ts, days, get_ata_balance, hours, init_ata, init_mint,
                initialize_multidelegate_action, minutes, move_clock_forward, setup,
                CreateDelegation, TransferDelegation,
            },
        },
        MultiDelegatorError,
    };
    use litesvm::LiteSVM;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction_error::TransactionError::InstructionError;
    use spl_token::instruction::TokenInstruction::{Approve, Revoke};

    fn setup_recurring_delegation(
        amount_per_period: u64,
        period_length_s: u64,
        start_ts: i64,
        expiry_ts: i64,
        nonce: u64,
    ) -> (
        LiteSVM,
        Keypair,
        Keypair,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
    ) {
        let (mut litesvm, alice) = setup();
        let bob = Keypair::new();
        litesvm.airdrop(&bob.pubkey(), 10_000_000).unwrap();

        let mint = init_mint(
            &mut litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(alice.pubkey()),
            &[],
        );
        let alice_ata = init_ata(&mut litesvm, mint, alice.pubkey(), 100_000_000);
        let bob_ata = init_ata(&mut litesvm, mint, bob.pubkey(), 0);

        let init_result = initialize_multidelegate_action(&mut litesvm, &alice, mint);
        init_result.0.assert_ok();

        let (res, delegation_pda) = CreateDelegation::new(&mut litesvm, &alice, mint, bob.pubkey())
            .nonce(nonce)
            .recurring(amount_per_period, period_length_s, start_ts, expiry_ts);
        res.assert_ok();

        (
            litesvm,
            alice,
            bob,
            delegation_pda,
            mint,
            alice_ata,
            bob_ata,
            init_result.1,
        )
    }

    #[test]
    fn test_recurring_transfer_success() {
        let amount_per_period: u64 = 50_000_000;
        let period_length_s: u64 = hours(1);
        let start_ts: i64 = current_ts();
        let expiry_ts: i64 = current_ts() + days(1) as i64;
        let nonce = 0;

        let (mut litesvm, alice, bob, delegation_pda, mint, _, bob_ata, _) =
            setup_recurring_delegation(
                amount_per_period,
                period_length_s,
                start_ts,
                expiry_ts,
                nonce,
            );

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);

        let transfer_amount: u64 = 10_000_000;
        TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
            .amount(transfer_amount)
            .recurring()
            .assert_ok();

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 10_000_000);

        let delegation_account = litesvm.get_account(&delegation_pda).unwrap();
        let delegation = RecurringDelegation::load(&delegation_account.data).unwrap();
        let delegation_amount_pulled_in_period = delegation.amount_pulled_in_period;
        let delegation_current_period_start_ts = delegation.current_period_start_ts;
        let delegation_period_length_s = delegation.period_length_s;
        assert_eq!(delegation_amount_pulled_in_period, 10_000_000);
        assert_eq!(delegation_current_period_start_ts, start_ts);
        assert_eq!(delegation_period_length_s, period_length_s);

        move_clock_forward(&mut litesvm, minutes(15));

        TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
            .amount(transfer_amount)
            .recurring()
            .assert_ok();

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 20_000_000);

        let delegation_account = litesvm.get_account(&delegation_pda).unwrap();
        let delegation = RecurringDelegation::load(&delegation_account.data).unwrap();
        let delegation_amount_pulled_in_period = delegation.amount_pulled_in_period;
        assert_eq!(delegation_amount_pulled_in_period, 20_000_000);

        move_clock_forward(&mut litesvm, minutes(15));

        TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
            .amount(transfer_amount)
            .recurring()
            .assert_ok();

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 30_000_000);

        let delegation_account = litesvm.get_account(&delegation_pda).unwrap();
        let delegation = RecurringDelegation::load(&delegation_account.data).unwrap();
        let delegation_amount_pulled_in_period = delegation.amount_pulled_in_period;
        assert_eq!(delegation_amount_pulled_in_period, 30_000_000);
    }

    #[test]
    fn test_recurring_transfer_exceeds_period_limit() {
        let amount_per_period: u64 = 50_000_000;
        let period_length_s: u64 = hours(1);
        let start_ts: i64 = current_ts();
        let expiry_ts: i64 = current_ts() + days(1) as i64;
        let nonce = 1;

        let (mut litesvm, alice, bob, delegation_pda, mint, _, bob_ata, _) =
            setup_recurring_delegation(
                amount_per_period,
                period_length_s,
                start_ts,
                expiry_ts,
                nonce,
            );

        move_clock_forward(&mut litesvm, period_length_s + 1);

        let transfer_amount: u64 = 60_000_000;
        let result =
            TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
                .amount(transfer_amount)
                .recurring();

        result.assert_err(MultiDelegatorError::AmountExceedsPeriodLimit);
        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);
    }

    #[test]
    fn test_recurring_transfer_expired() {
        let amount_per_period: u64 = 50_000_000;
        let period_length_s: u64 = hours(1);
        let start_ts: i64 = current_ts();
        let expiry_ts: i64 = current_ts() + days(1) as i64;
        let nonce = 1;

        let (mut litesvm, alice, bob, delegation_pda, mint, _, bob_ata, _) =
            setup_recurring_delegation(
                amount_per_period,
                period_length_s,
                start_ts,
                expiry_ts,
                nonce,
            );

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);

        let transfer_amount: u64 = 30_000_000;
        let result =
            TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
                .amount(transfer_amount)
                .recurring();
        result.assert_ok();
        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 30_000_000);

        // Now let's move the clock and try to transfer again
        move_clock_forward(&mut litesvm, (current_ts() + (days(2) as i64)) as u64);

        let transfer_amount: u64 = 30_000_000;
        let result =
            TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
                .amount(transfer_amount)
                .recurring();
        result.assert_err(MultiDelegatorError::DelegationExpired);
        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 30_000_000);
    }

    #[test]
    fn test_recurring_transfer_multiple_periods() {
        let amount_per_period: u64 = 50_000_000;
        let period_length_s: u64 = hours(1);
        let start_ts: i64 = current_ts();
        let expiry_ts: i64 = current_ts() + days(1) as i64;
        let nonce = 1;

        let (mut litesvm, alice, bob, delegation_pda, mint, _, bob_ata, _) =
            setup_recurring_delegation(
                amount_per_period,
                period_length_s,
                start_ts,
                expiry_ts,
                nonce,
            );

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);

        TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
            .amount(30_000_000)
            .recurring()
            .assert_ok();

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 30_000_000);

        let delegation_account = litesvm.get_account(&delegation_pda).unwrap();
        let delegation_amount_pulled_in_period =
            RecurringDelegation::load(&delegation_account.data)
                .unwrap()
                .amount_pulled_in_period;
        assert_eq!(delegation_amount_pulled_in_period, 30_000_000);

        // Move forward until new time period
        move_clock_forward(&mut litesvm, period_length_s);

        TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
            .amount(30_000_000)
            .recurring()
            .assert_ok();

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 60_000_000);

        let delegation_account = litesvm.get_account(&delegation_pda).unwrap();
        let delegation_amount_pulled_in_period =
            RecurringDelegation::load(&delegation_account.data)
                .unwrap()
                .amount_pulled_in_period;
        assert_eq!(delegation_amount_pulled_in_period, 30_000_000);
    }

    #[test]
    fn test_recurring_transfer_skip_multiple_periods() {
        let amount_per_period: u64 = 50_000_000;
        let period_length_s: u64 = hours(1);
        let start_ts: i64 = current_ts();
        let expiry_ts: i64 = current_ts() + days(1) as i64;
        let nonce = 2;

        let (mut litesvm, alice, bob, delegation_pda, mint, _, bob_ata, _) =
            setup_recurring_delegation(
                amount_per_period,
                period_length_s,
                start_ts,
                expiry_ts,
                nonce,
            );

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);

        // Period 0: Transfer 10M
        TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
            .amount(10_000_000)
            .recurring()
            .assert_ok();
        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 10_000_000);

        // Move forward 3 periods
        move_clock_forward(&mut litesvm, period_length_s * 3);

        // Period 3: Transfer 10M
        TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
            .amount(10_000_000)
            .recurring()
            .assert_ok();

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 20_000_000);

        let delegation_account = litesvm.get_account(&delegation_pda).unwrap();
        let delegation = RecurringDelegation::load(&delegation_account.data).unwrap();

        // New start should be start_ts + 3 * period
        let expected_start = start_ts + (period_length_s * 3) as i64;
        let actual_start = delegation.current_period_start_ts;
        let actual_pulled = delegation.amount_pulled_in_period;

        assert_eq!(actual_start, expected_start);
        assert_eq!(actual_pulled, 10_000_000);
    }

    #[test]
    fn test_recurring_transfer_skip_period_cannot_double_claim() {
        // Bug hypothesis: after skipping one period with no claims, the delegatee
        // can claim twice (2x amount_per_period) in the next period.
        //
        // Scenario:
        //   Period 0: claim full allowance
        //   Period 1: no claims (skipped)
        //   Period 2 start: claim full allowance, then immediately try again
        //
        // Expected: second claim in period 2 should fail — skipped periods
        // do not accumulate allowance.
        let amount_per_period: u64 = 50_000_000;
        let period_length_s: u64 = hours(1);
        let start_ts: i64 = current_ts();
        let expiry_ts: i64 = current_ts() + days(7) as i64;
        let nonce = 3;

        let (mut litesvm, alice, bob, delegation_pda, mint, _, bob_ata, _) =
            setup_recurring_delegation(
                amount_per_period,
                period_length_s,
                start_ts,
                expiry_ts,
                nonce,
            );

        // Period 0: Use the full allowance
        TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
            .amount(amount_per_period)
            .recurring()
            .assert_ok();
        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 50_000_000);

        // Skip period 1 entirely — advance exactly to the start of period 2
        move_clock_forward(&mut litesvm, period_length_s * 2);

        // Period 2, transfer 1: claim full allowance — should succeed
        TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
            .amount(amount_per_period)
            .recurring()
            .assert_ok();
        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 100_000_000);

        // Period 2, transfer 2: immediately try to claim again — should fail
        let result =
            TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
                .amount(amount_per_period)
                .recurring();
        result.assert_err(MultiDelegatorError::AmountExceedsPeriodLimit);

        // Balances unchanged after failed transfer
        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 100_000_000);

        // Verify delegation state
        let delegation_account = litesvm.get_account(&delegation_pda).unwrap();
        let delegation = RecurringDelegation::load(&delegation_account.data).unwrap();
        let actual_pulled = delegation.amount_pulled_in_period;
        let actual_start = delegation.current_period_start_ts;
        assert_eq!(actual_pulled, amount_per_period);
        let expected_start = start_ts + (period_length_s * 2) as i64;
        assert_eq!(actual_start, expected_start);
    }

    #[test]
    fn test_recurring_transfer_delegator_mismatch_exploit() {
        // This test demonstrates the access control vulnerability where an attacker
        // can use their own delegation to transfer funds from another user's account

        let amount_per_period: u64 = 50_000_000;
        let period_length_s: u64 = hours(1);
        let start_ts: i64 = current_ts();
        let expiry_ts: i64 = current_ts() + days(1) as i64;
        let nonce = 0;

        // Setup: Alice (victim) with funds and Bob (attacker)
        let (mut litesvm, alice, bob, _alice_delegation_pda, mint, alice_ata, bob_ata, _) =
            setup_recurring_delegation(
                amount_per_period,
                period_length_s,
                start_ts,
                expiry_ts,
                nonce,
            );

        initialize_multidelegate_action(&mut litesvm, &bob, mint)
            .0
            .assert_ok();

        // Attacker (Bob) creates a self-delegation (Bob -> Bob) with a large allowance
        let (_res, bob_delegation_pda) =
            CreateDelegation::new(&mut litesvm, &bob, mint, bob.pubkey())
                .nonce(nonce)
                .recurring(1_000_000_000, period_length_s, start_ts, expiry_ts);
        _res.assert_ok();

        let transfer_amount: u64 = 30_000_000;

        // Exploit: Attacker tries to transfer from Alice's ATA using their own delegation
        // by passing Alice's delegator_pubkey in the instruction data
        let result =
            TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, bob_delegation_pda)
                .amount(transfer_amount)
                .to(bob_ata)
                .recurring();

        // After the fix, this should fail with Unauthorized error
        result.assert_err(MultiDelegatorError::Unauthorized);

        // Verify Alice's funds are untouched
        assert_eq!(get_ata_balance(&litesvm, &alice_ata), 100_000_000);
        // Verify Bob received no funds
        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);
    }

    #[test]
    fn test_recurring_transfer_token_revoke() {
        let amount_per_period: u64 = 50_000_000;
        let period_length_s: u64 = hours(1);
        let start_ts: i64 = current_ts();
        let expiry_ts: i64 = current_ts() + days(1) as i64;
        let nonce = 0;

        let (mut litesvm, alice, bob, delegation_pda, mint, alice_ata, bob_ata, multidelegate_pda) =
            setup_recurring_delegation(
                amount_per_period,
                period_length_s,
                start_ts,
                expiry_ts,
                nonce,
            );

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);

        TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
            .amount(50_000_000)
            .recurring()
            .assert_ok();
        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 50_000_000);

        // Let's revoke the token approval
        let ix = Instruction {
            program_id: TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(alice_ata, false),
                AccountMeta::new(alice.pubkey(), true),
            ],
            data: Revoke.pack(),
        };
        assert!(build_and_send_transaction(&mut litesvm, &[&alice], &alice.pubkey(), &ix).is_ok());

        // Now let's move the clock and try to fetch recurring delegation again
        move_clock_forward(&mut litesvm, period_length_s);

        // Now, let's try again
        let result =
            TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
                .amount(50_000_000)
                .recurring();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().err,
            InstructionError(
                0,
                solana_instruction::error::InstructionError::Custom(
                    spl_token::error::TokenError::OwnerMismatch as u32
                ),
            )
        );

        // Doing approval once again fixes it, but it has to be max possible for it to work

        // Scenario 1: We approve, but less amount
        let ix = Instruction {
            program_id: TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(alice_ata, false),
                AccountMeta::new(multidelegate_pda, false),
                AccountMeta::new(alice.pubkey(), true),
            ],
            data: Approve { amount: 100000 }.pack(),
        };
        assert!(build_and_send_transaction(&mut litesvm, &[&alice], &alice.pubkey(), &ix).is_ok());

        // Since the approval amount is less than what is needed, we fail again
        let result =
            TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
                .amount(50_000_000)
                .recurring();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().err,
            InstructionError(
                0,
                solana_instruction::error::InstructionError::Custom(
                    spl_token::error::TokenError::InsufficientFunds as u32
                ),
            )
        );

        // Scenario 2: We approve for max amount. Now it should work as usual
        let ix = Instruction {
            program_id: TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(alice_ata, false),
                AccountMeta::new(multidelegate_pda, false),
                AccountMeta::new(alice.pubkey(), true),
            ],
            data: Approve { amount: u64::MAX }.pack(),
        };
        assert!(build_and_send_transaction(&mut litesvm, &[&alice], &alice.pubkey(), &ix).is_ok());

        TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
            .amount(50_000_000)
            .recurring()
            .assert_ok();
    }
}
