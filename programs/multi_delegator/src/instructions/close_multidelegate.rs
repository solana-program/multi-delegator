use pinocchio::{error::ProgramError, AccountView, ProgramResult};

use crate::{
    AccountCheck, AccountClose, MultiDelegate, MultiDelegatorError, ProgramAccount, SignerAccount,
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

    if multi_delegate.user.as_ref() != accounts.user.address().as_ref() {
        return Err(MultiDelegatorError::Unauthorized.into());
    }

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
