use pinocchio::{error::ProgramError, AccountView, ProgramResult};

use crate::{
    AccountCheck, AccountClose, MultiDelegate, MultiDelegatorError, ProgramAccount, SignerAccount,
    WritableAccount,
};

pub struct CloseMultiDelegateAccounts<'a> {
    pub user: &'a AccountView,
    pub multi_delegate: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for CloseMultiDelegateAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [user, multi_delegate] = accounts else {
            return Err(MultiDelegatorError::NotEnoughAccountKeys.into());
        };

        SignerAccount::check(user)?;
        WritableAccount::check(user)?;
        WritableAccount::check(multi_delegate)?;
        ProgramAccount::check(multi_delegate)?;

        Ok(Self {
            user,
            multi_delegate,
        })
    }
}

pub const DISCRIMINATOR: &u8 = &6;

/// Closes a MultiDelegate PDA account, returning the lamports to the user.
/// Only the user who owns the MultiDelegate can close it.
pub fn process(accounts: &[AccountView]) -> ProgramResult {
    let accounts = CloseMultiDelegateAccounts::try_from(accounts)?;

    let data = accounts.multi_delegate.try_borrow()?;
    let multi_delegate = MultiDelegate::load(&data)?;

    multi_delegate.check_owner(accounts.user.address())?;

    // Verify the PDA derivation matches
    let expected_pda = MultiDelegate::verify_pda(
        &multi_delegate.user,
        &multi_delegate.token_mint,
        multi_delegate.bump,
    )?;
    if expected_pda.as_ref() != accounts.multi_delegate.address().as_ref() {
        return Err(MultiDelegatorError::InvalidMultiDelegatePda.into());
    }

    drop(data);

    ProgramAccount::close(accounts.multi_delegate, accounts.user)
}

#[cfg(test)]
mod tests {
    use solana_signer::Signer;

    use crate::{
        tests::{
            asserts::TransactionResultExt,
            constants::{MINT_DECIMALS, TOKEN_PROGRAM_ID},
            utils::{
                init_ata, init_mint, init_wallet, initialize_multidelegate_action, setup,
                CloseMultiDelegate,
            },
        },
        MultiDelegatorError,
    };

    #[test]
    fn close_multidelegate() {
        let (litesvm, user) = &mut setup();

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(user.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, user.pubkey(), 1_000_000);

        let (res, multi_delegate_pda, _bump) = initialize_multidelegate_action(litesvm, user, mint);
        res.assert_ok();

        let account_before = litesvm.get_account(&multi_delegate_pda);
        assert!(account_before.is_some());
        let rent = account_before.unwrap().lamports;

        let user_balance_before = litesvm.get_account(&user.pubkey()).unwrap().lamports;

        let res = CloseMultiDelegate::new(litesvm, user, mint).execute();
        res.assert_ok();

        let account_after = litesvm.get_account(&multi_delegate_pda);
        assert!(
            account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0
        );

        let user_balance_after = litesvm.get_account(&user.pubkey()).unwrap().lamports;
        assert!(user_balance_after > user_balance_before);
        assert!(user_balance_after >= user_balance_before + rent - 10000);
    }

    #[test]
    fn non_owner_cannot_close() {
        let (litesvm, user) = &mut setup();

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(user.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, user.pubkey(), 1_000_000);

        let (res, multi_delegate_pda, _bump) = initialize_multidelegate_action(litesvm, user, mint);
        res.assert_ok();

        let attacker = init_wallet(litesvm, 1_000_000_000);
        let res = CloseMultiDelegate::new(litesvm, &attacker, mint)
            .pda(multi_delegate_pda)
            .execute();
        res.assert_err(MultiDelegatorError::Unauthorized);

        // Account should still exist
        let account_after = litesvm.get_account(&multi_delegate_pda);
        assert!(account_after.is_some());
        assert!(account_after.as_ref().map(|a| a.lamports).unwrap_or(0) > 0);
    }

    #[test]
    fn writable_accounts_must_be_writable() {
        use solana_instruction::{AccountMeta, Instruction};

        use crate::{
            instructions::close_multidelegate,
            tests::{
                constants::PROGRAM_ID,
                idl,
                utils::{build_and_send_transaction, init_wallet},
            },
        };

        let writable = idl::writable_account_indices("closeMultiDelegate");

        let (litesvm, user) = &mut setup();
        let fee_payer = init_wallet(litesvm, 10_000_000_000);

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(user.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, user.pubkey(), 1_000_000);

        let (res, multi_delegate_pda, _) = initialize_multidelegate_action(litesvm, user, mint);
        res.assert_ok();

        for (idx, _name, is_signer) in &writable {
            let mut accounts = vec![
                AccountMeta::new(user.pubkey(), true),
                AccountMeta::new(multi_delegate_pda, false),
            ];

            // Flip writable account to readonly, preserving signer flag
            let pubkey = accounts[*idx].pubkey;
            accounts[*idx] = AccountMeta::new_readonly(pubkey, *is_signer);

            let ix = Instruction {
                program_id: PROGRAM_ID,
                accounts,
                data: vec![*close_multidelegate::DISCRIMINATOR],
            };

            let res =
                build_and_send_transaction(litesvm, &[&fee_payer, user], &fee_payer.pubkey(), &ix);
            res.assert_err(MultiDelegatorError::AccountNotWritable);
        }
    }

    #[test]
    fn signer_accounts_must_be_signers() {
        use solana_instruction::{AccountMeta, Instruction};

        use crate::{
            instructions::close_multidelegate,
            tests::{
                constants::PROGRAM_ID,
                idl,
                utils::{build_and_send_transaction, init_wallet},
            },
        };

        let signers = idl::signer_account_indices("closeMultiDelegate");

        let (litesvm, user) = &mut setup();
        let fee_payer = init_wallet(litesvm, 10_000_000_000);

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(user.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, user.pubkey(), 1_000_000);

        let (res, multi_delegate_pda, _) = initialize_multidelegate_action(litesvm, user, mint);
        res.assert_ok();

        for (idx, _name, is_writable) in &signers {
            let mut accounts = vec![
                AccountMeta::new(user.pubkey(), true),
                AccountMeta::new(multi_delegate_pda, false),
            ];

            // Flip signer to non-signer, preserving writable flag
            let pubkey = accounts[*idx].pubkey;
            accounts[*idx] = if *is_writable {
                AccountMeta::new(pubkey, false)
            } else {
                AccountMeta::new_readonly(pubkey, false)
            };

            let ix = Instruction {
                program_id: PROGRAM_ID,
                accounts,
                data: vec![*close_multidelegate::DISCRIMINATOR],
            };

            let res = build_and_send_transaction(litesvm, &[&fee_payer], &fee_payer.pubkey(), &ix);
            res.assert_err(MultiDelegatorError::NotSigner);
        }
    }

    #[test]
    fn closed_account_is_zeroed() {
        let (litesvm, user) = &mut setup();

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(user.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, user.pubkey(), 1_000_000);

        let (res, multi_delegate_pda, _bump) = initialize_multidelegate_action(litesvm, user, mint);
        res.assert_ok();

        let res = CloseMultiDelegate::new(litesvm, user, mint).execute();
        res.assert_ok();

        let account_after = litesvm.get_account(&multi_delegate_pda);
        if let Some(account) = account_after {
            assert!(
                account.data.iter().all(|&byte| byte == 0),
                "All data should be zeroed after close"
            );
        }
    }
}
