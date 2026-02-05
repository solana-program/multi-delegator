use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    ProgramResult,
};

use crate::{
    helpers::{transfer_with_delegate, TransferAccounts, TransferData},
    state::{DelegationKind, FixedDelegation, KIND_OFFSET},
    AccountCheck, MultiDelegateAccount, MultiDelegatorError, ProgramAccount, SignerAccount,
    TokenAccountInterface,
};

pub const DISCRIMINATOR: &u8 = &4;

pub fn process(accounts: &[AccountInfo], transfer: &TransferData) -> ProgramResult {
    let accounts_struct = FixedTransferAccounts::try_from(accounts)?;

    // Validate kind matches Fixed
    {
        let binding = accounts_struct.delegation_pda.try_borrow_data()?;

        if binding.len() <= KIND_OFFSET {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }

        let kind = binding[KIND_OFFSET];
        if kind != DelegationKind::Fixed as u8 {
            return Err(MultiDelegatorError::TransferKindMismatch.into());
        }

        let delegation = FixedDelegation::load(&binding)?;

        // Fail fast: Check authorization first
        if delegation.header.delegatee != *accounts_struct.delegatee.key() {
            return Err(MultiDelegatorError::Unauthorized.into());
        }

        let current_ts = Clock::get()?.unix_timestamp;

        if current_ts > delegation.expiry_ts {
            return Err(MultiDelegatorError::DelegationExpired.into());
        }

        if transfer.amount > delegation.amount {
            return Err(MultiDelegatorError::AmountExceedsLimit.into());
        }
    }

    transfer_with_delegate(
        transfer.amount,
        &transfer.delegator,
        &transfer.mint,
        &TransferAccounts {
            delegator_ata: accounts_struct.delegator_ata,
            to_ata: accounts_struct.receiver_ata,
            multidelegate_pda: accounts_struct.multi_delegate,
            token_program: accounts_struct.token_program,
        },
    )?;

    let mut binding = accounts_struct.delegation_pda.try_borrow_mut_data()?;
    let delegation_mut = FixedDelegation::load_mut(&mut binding)?;
    delegation_mut.amount -= transfer.amount;

    Ok(())
}

pub struct FixedTransferAccounts<'a> {
    pub delegation_pda: &'a AccountInfo,
    pub multi_delegate: &'a AccountInfo,
    pub delegator_ata: &'a AccountInfo,
    pub receiver_ata: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
    pub delegatee: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for FixedTransferAccounts<'a> {
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
        state::FixedDelegation,
        tests::{
            asserts::assert_error,
            constants::{MINT_DECIMALS, TOKEN_PROGRAM_ID},
            utils::{
                current_ts, days, get_ata_balance, init_ata, init_mint,
                initialize_multidelegate_action, setup, CreateDelegation, TransferDelegation,
            },
        },
        MultiDelegatorError,
    };
    use litesvm::LiteSVM;
    use solana_keypair::Keypair;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;

    fn setup_fixed_delegation(
        amount: u64,
        expiry_ts: i64,
        nonce: u64,
    ) -> (LiteSVM, Keypair, Keypair, Pubkey, Pubkey, Pubkey, Pubkey) {
        let (mut lite_svm, alice) = setup();
        let bob = Keypair::new();
        lite_svm.airdrop(&bob.pubkey(), 1_000_000).unwrap();

        let mint = init_mint(
            &mut lite_svm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(alice.pubkey()),
        );
        let alice_ata = init_ata(&mut lite_svm, mint, alice.pubkey(), 100_000_000);
        let bob_ata = init_ata(&mut lite_svm, mint, bob.pubkey(), 0);

        initialize_multidelegate_action(&mut lite_svm, &alice, mint)
            .0
            .unwrap();

        let (res, delegation_pda) =
            CreateDelegation::new(&mut lite_svm, &alice, mint, bob.pubkey())
                .nonce(nonce)
                .fixed(amount, expiry_ts);
        res.unwrap();

        (
            lite_svm,
            alice,
            bob,
            delegation_pda,
            mint,
            alice_ata,
            bob_ata,
        )
    }

    #[test]
    fn test_fixed_transfer_success() {
        let amount: u64 = 50_000_000;
        let expiry_ts: i64 = current_ts() + days(1) as i64;
        let nonce = 0;

        let (mut litesvm, alice, bob, delegation_pda, mint, _alice_ata, bob_ata) =
            setup_fixed_delegation(amount, expiry_ts, nonce);

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);

        let transfer_amount: u64 = 30_000_000;
        TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
            .amount(transfer_amount)
            .fixed()
            .unwrap();

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 30_000_000);

        let delegation_account = litesvm.get_account(&delegation_pda).unwrap();
        let delegation = FixedDelegation::load(&delegation_account.data).unwrap();
        let del_amount = delegation.amount;
        let del_expiry_s = delegation.expiry_ts;
        assert_eq!(del_amount, 20_000_000);
        assert_eq!(del_expiry_s, expiry_ts);
    }

    #[test]
    fn test_fixed_transfer_multiple_times() {
        let amount: u64 = 50_000_000;
        let expiry_s: i64 = current_ts() + days(1) as i64;
        let nonce = 1;

        let (mut litesvm, alice, bob, delegation_pda, mint, _, bob_ata) =
            setup_fixed_delegation(amount, expiry_s, nonce);

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);

        let transfer_amount: u64 = 30_000_000;
        TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
            .amount(transfer_amount)
            .fixed()
            .unwrap();

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 30_000_000);

        let delegation_account = litesvm.get_account(&delegation_pda).unwrap();
        let del_amount = FixedDelegation::load(&delegation_account.data)
            .unwrap()
            .amount;
        assert_eq!(del_amount, 20_000_000);

        let result =
            TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
                .amount(transfer_amount)
                .fixed();

        assert_error(result, MultiDelegatorError::AmountExceedsLimit);
        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 30_000_000);

        let delegation_account = litesvm.get_account(&delegation_pda).unwrap();
        let del_amount = FixedDelegation::load(&delegation_account.data)
            .unwrap()
            .amount;
        assert_eq!(del_amount, 20_000_000);
    }

    #[test]
    fn test_fixed_transfer_exceeds_amount() {
        let amount: u64 = 50_000_000;
        let expiry_ts: i64 = current_ts() + days(1) as i64;
        let nonce = 1;

        let (mut litesvm, alice, bob, delegation_pda, mint, _, bob_ata) =
            setup_fixed_delegation(amount, expiry_ts, nonce);

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);

        let transfer_amount: u64 = 60_000_000;
        let result =
            TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
                .amount(transfer_amount)
                .fixed();

        // Check that the error matches AmountExceedsLimit
        assert_error(result, MultiDelegatorError::AmountExceedsLimit);

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);

        let delegation_account = litesvm.get_account(&delegation_pda).unwrap();
        let del_amount = FixedDelegation::load(&delegation_account.data)
            .unwrap()
            .amount;
        assert_eq!(del_amount, 50_000_000);
    }

    #[test]
    fn test_fixed_transfer_expired() {
        let amount: u64 = 50_000_000;
        let expiry_ts: i64 = current_ts() - days(1) as i64;
        let nonce = 1;

        let (mut litesvm, alice, bob, delegation_pda, mint, _, bob_ata) =
            setup_fixed_delegation(amount, expiry_ts, nonce);

        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);

        let transfer_amount: u64 = 30_000_000;
        let result =
            TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
                .amount(transfer_amount)
                .fixed();

        assert_error(result, MultiDelegatorError::DelegationExpired);
        assert_eq!(get_ata_balance(&litesvm, &bob_ata), 0);

        let delegation_account = litesvm.get_account(&delegation_pda).unwrap();
        let delegation_amount = FixedDelegation::load(&delegation_account.data)
            .unwrap()
            .amount;
        assert_eq!(delegation_amount, 50_000_000);
    }

    #[test]
    fn test_fixed_transfer_wrong_signer() {
        let amount: u64 = 50_000_000;
        let expiry_ts: i64 = current_ts() + days(1) as i64;
        let nonce = 10;

        let (mut litesvm, alice, _bob, delegation_pda, mint, _, bob_ata) =
            setup_fixed_delegation(amount, expiry_ts, nonce);

        // Eve is the attacker
        let eve = Keypair::new();
        litesvm.airdrop(&eve.pubkey(), 1_000_000).unwrap();

        let transfer_amount: u64 = 10_000_000;

        // Use the new helper
        let result =
            TransferDelegation::new(&mut litesvm, &eve, alice.pubkey(), mint, delegation_pda)
                .amount(transfer_amount)
                .to(bob_ata)
                .fixed();

        // Expect Unauthorized error
        assert_error(result, MultiDelegatorError::Unauthorized);
    }

    #[test]
    fn test_fixed_transfer_to_third_party() {
        let amount: u64 = 50_000_000;
        let expiry_ts: i64 = current_ts() + days(1) as i64;
        let nonce = 0;

        // Alice delegates to Bob
        let (mut litesvm, alice, bob, delegation_pda, mint, _, _) =
            setup_fixed_delegation(amount, expiry_ts, nonce);

        // Charlie is a third party
        let charlie = Keypair::new();
        let charlie_ata = init_ata(&mut litesvm, mint, charlie.pubkey(), 0);

        let transfer_amount: u64 = 10_000_000;

        // Bob transfers from Alice -> Charlie
        TransferDelegation::new(&mut litesvm, &bob, alice.pubkey(), mint, delegation_pda)
            .amount(transfer_amount)
            .to(charlie_ata)
            .fixed()
            .unwrap();

        // Verify Charlie received funds
        assert_eq!(get_ata_balance(&litesvm, &charlie_ata), 10_000_000);
    }
}
