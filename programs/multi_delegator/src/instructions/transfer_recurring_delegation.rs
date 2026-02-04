use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    ProgramResult,
};

use crate::{
    helpers::{transfer_with_delegate, TransferAccounts, TransferData},
    state::{DelegationKind, RecurringDelegation},
    AccountCheck, MultiDelegateAccount, MultiDelegatorError, ProgramAccount, SignerAccount,
    TokenAccountInterface,
};

pub const DISCRIMINATOR: &u8 = &5;

pub fn process(accounts: &[AccountInfo], transfer_data: &TransferData) -> ProgramResult {
    let accounts_struct = RecurringTransferAccounts::try_from(accounts)?;

    let current_ts = Clock::get()?.unix_timestamp;
    {
        let mut binding = accounts_struct.delegation_pda.try_borrow_mut_data()?;
        let delegation_mut = RecurringDelegation::load_mut(&mut binding)?;

        if delegation_mut.header.kind != DelegationKind::Recurring as u8 {
            return Err(MultiDelegatorError::TransferKindMismatch.into());
        }

        // Fail fast: Check authorization first
        if delegation_mut.header.delegatee != *accounts_struct.delegatee.key() {
            return Err(MultiDelegatorError::Unauthorized.into());
        }

        if current_ts > delegation_mut.expiry_ts {
            return Err(MultiDelegatorError::DelegationExpired.into());
        }

        // If we have passed into the next period, then start new period
        let time_since_start = current_ts.saturating_sub(delegation_mut.current_period_start_ts);
        let period_length = delegation_mut.period_length_s as i64;

        if time_since_start >= period_length {
            let periods_passed = time_since_start / period_length;
            delegation_mut.current_period_start_ts += periods_passed * period_length;
            delegation_mut.amount_pulled_in_period = 0;
        }

        let available = delegation_mut.amount_per_period - delegation_mut.amount_pulled_in_period;
        if transfer_data.amount > available {
            return Err(MultiDelegatorError::AmountExceedsPeriodLimit.into());
        }

        delegation_mut.amount_pulled_in_period += transfer_data.amount;
    }

    transfer_with_delegate(
        transfer_data.amount,
        &transfer_data.delegator,
        &transfer_data.mint,
        &TransferAccounts {
            delegator_ata: accounts_struct.delegator_ata,
            to_ata: accounts_struct.receiver_ata,
            multidelegate_pda: accounts_struct.multi_delegate,
        },
    )?;

    Ok(())
}

pub struct RecurringTransferAccounts<'a> {
    pub delegation_pda: &'a AccountInfo,
    pub multi_delegate: &'a AccountInfo,
    pub delegator_ata: &'a AccountInfo,
    pub receiver_ata: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
    pub delegatee: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for RecurringTransferAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [delegation_pda, multi_delegate, delegator_ata, receiver_ata, token_program, delegatee, ..] =
            accounts
        else {
            return Err(MultiDelegatorError::NotEnoughAccountKeys.into());
        };

        ProgramAccount::check(delegation_pda)?;
        MultiDelegateAccount::check(multi_delegate)?;
        TokenAccountInterface::check(delegator_ata)?;
        TokenAccountInterface::check(receiver_ata)?;
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
    use crate::{
        state::RecurringDelegation,
        tests::{
            asserts::assert_error,
            constants::{MINT_DECIMALS, TOKEN_PROGRAM_ID},
            utils::{
                create_recurring_delegation_action, current_ts, days, get_ata_balance, hours,
                init_ata, init_mint, initialize_multidelegate_action, minutes, move_clock_forward,
                setup, transfer_recurring_action,
            },
        },
        MultiDelegatorError,
    };
    use litesvm::LiteSVM;
    use solana_keypair::Keypair;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;

    fn setup_recurring_delegation(
        amount_per_period: u64,
        period_length_s: u64,
        start_ts: i64,
        expiry_ts: i64,
        nonce: u64,
    ) -> (LiteSVM, Keypair, Keypair, Pubkey, Pubkey, Pubkey, Pubkey) {
        let (mut litesvm, alice) = setup();
        let bob = Keypair::new();
        litesvm.airdrop(&bob.pubkey(), 1_000_000).unwrap();

        let mint = init_mint(
            &mut litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(alice.pubkey()),
        );
        let alice_ata = init_ata(&mut litesvm, mint, alice.pubkey(), 100_000_000);
        let bob_ata = init_ata(&mut litesvm, mint, bob.pubkey(), 0);

        initialize_multidelegate_action(&mut litesvm, &alice, mint)
            .0
            .unwrap();

        let (res, delegation_pda) = create_recurring_delegation_action(
            &mut litesvm,
            &alice,
            mint,
            bob.pubkey(),
            nonce,
            amount_per_period,
            period_length_s,
            start_ts,
            expiry_ts,
        );
        res.unwrap();

        (
            litesvm,
            alice,
            bob,
            delegation_pda,
            mint,
            alice_ata,
            bob_ata,
        )
    }

    #[test]
    fn test_recurring_transfer_success() {
        let amount_per_period: u64 = 50_000_000;
        let period_length_s: u64 = hours(1);
        let start_ts: i64 = current_ts();
        let expiry_ts: i64 = current_ts() + days(1) as i64;
        let nonce = 0;

        let (mut litesvm, alice, bob, delegation_pda, mint, _, bob_ata) =
            setup_recurring_delegation(
                amount_per_period,
                period_length_s,
                start_ts,
                expiry_ts,
                nonce,
            );

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);

        let transfer_amount: u64 = 10_000_000;
        transfer_recurring_action(
            &mut litesvm,
            &alice,
            &bob,
            mint,
            delegation_pda,
            transfer_amount,
        )
        .unwrap();

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

        transfer_recurring_action(
            &mut litesvm,
            &alice,
            &bob,
            mint,
            delegation_pda,
            transfer_amount,
        )
        .unwrap();

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 20_000_000);

        let delegation_account = litesvm.get_account(&delegation_pda).unwrap();
        let delegation = RecurringDelegation::load(&delegation_account.data).unwrap();
        let delegation_amount_pulled_in_period = delegation.amount_pulled_in_period;
        assert_eq!(delegation_amount_pulled_in_period, 20_000_000);

        move_clock_forward(&mut litesvm, minutes(15));

        transfer_recurring_action(
            &mut litesvm,
            &alice,
            &bob,
            mint,
            delegation_pda,
            transfer_amount,
        )
        .unwrap();

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

        let (mut litesvm, alice, bob, delegation_pda, mint, _, bob_ata) =
            setup_recurring_delegation(
                amount_per_period,
                period_length_s,
                start_ts,
                expiry_ts,
                nonce,
            );

        move_clock_forward(&mut litesvm, period_length_s + 1);

        let transfer_amount: u64 = 60_000_000;
        let result = transfer_recurring_action(
            &mut litesvm,
            &alice,
            &bob,
            mint,
            delegation_pda,
            transfer_amount,
        );

        assert_error(result, MultiDelegatorError::AmountExceedsPeriodLimit);
        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);
    }

    #[test]
    fn test_recurring_transfer_expired() {
        let amount_per_period: u64 = 50_000_000;
        let period_length_s: u64 = hours(1);
        let start_ts: i64 = current_ts();
        let expiry_ts: i64 = current_ts() - days(1) as i64;
        let nonce = 1;

        let (mut litesvm, alice, bob, delegation_pda, mint, _, bob_ata) =
            setup_recurring_delegation(
                amount_per_period,
                period_length_s,
                start_ts,
                expiry_ts,
                nonce,
            );

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);

        let transfer_amount: u64 = 30_000_000;
        let result = transfer_recurring_action(
            &mut litesvm,
            &alice,
            &bob,
            mint,
            delegation_pda,
            transfer_amount,
        );

        assert_error(result, MultiDelegatorError::DelegationExpired);
        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);
    }

    #[test]
    fn test_recurring_transfer_multiple_periods() {
        let amount_per_period: u64 = 50_000_000;
        let period_length_s: u64 = hours(1);
        let start_ts: i64 = current_ts();
        let expiry_ts: i64 = current_ts() + days(1) as i64;
        let nonce = 1;

        let (mut litesvm, alice, bob, delegation_pda, mint, _, bob_ata) =
            setup_recurring_delegation(
                amount_per_period,
                period_length_s,
                start_ts,
                expiry_ts,
                nonce,
            );

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);

        transfer_recurring_action(&mut litesvm, &alice, &bob, mint, delegation_pda, 30_000_000)
            .unwrap();

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 30_000_000);

        let delegation_account = litesvm.get_account(&delegation_pda).unwrap();
        let delegation_amount_pulled_in_period =
            RecurringDelegation::load(&delegation_account.data)
                .unwrap()
                .amount_pulled_in_period;
        assert_eq!(delegation_amount_pulled_in_period, 30_000_000);

        // Move forward until new time period
        move_clock_forward(&mut litesvm, period_length_s);

        transfer_recurring_action(&mut litesvm, &alice, &bob, mint, delegation_pda, 30_000_000)
            .unwrap();

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

        let (mut litesvm, alice, bob, delegation_pda, mint, _, bob_ata) =
            setup_recurring_delegation(
                amount_per_period,
                period_length_s,
                start_ts,
                expiry_ts,
                nonce,
            );

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);

        // Period 0: Transfer 10M
        transfer_recurring_action(&mut litesvm, &alice, &bob, mint, delegation_pda, 10_000_000)
            .unwrap();
        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 10_000_000);

        // Move forward 3 periods
        move_clock_forward(&mut litesvm, period_length_s * 3);

        // Period 3: Transfer 10M
        transfer_recurring_action(&mut litesvm, &alice, &bob, mint, delegation_pda, 10_000_000)
            .unwrap();

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
}
