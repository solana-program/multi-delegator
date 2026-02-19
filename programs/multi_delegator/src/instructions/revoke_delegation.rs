use pinocchio::{error::ProgramError, AccountView, ProgramResult};

use crate::{
    AccountCheck, AccountClose, Header, MultiDelegatorError, ProgramAccount, SignerAccount,
    DELEGATEE_OFFSET, DELEGATOR_OFFSET, PAYER_OFFSET,
};

pub struct RevokeDelegationAccounts<'a> {
    pub authority: &'a AccountView,
    pub delegation_account: &'a AccountView,
    pub receiver: Option<&'a AccountView>,
}

impl<'a> TryFrom<&'a [AccountView]> for RevokeDelegationAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, delegation_account, rem @ ..] = accounts else {
            return Err(MultiDelegatorError::NotEnoughAccountKeys.into());
        };

        SignerAccount::check(authority)?;
        ProgramAccount::check(delegation_account)?;

        Ok(Self {
            authority,
            delegation_account,
            receiver: rem.first(),
        })
    }
}

pub const DISCRIMINATOR: &u8 = &3;

/// Revokes a delegation by closing the delegation PDA.
/// The rent lamports are returned to the original payer.
pub fn process(accounts: &[AccountView]) -> ProgramResult {
    let accounts = RevokeDelegationAccounts::try_from(accounts)?;

    let destination = {
        let data = accounts.delegation_account.try_borrow()?;

        if data.len() < Header::LEN {
            return Err(MultiDelegatorError::InvalidHeaderData.into());
        }

        let delegator_bytes: &[u8; 32] = data[DELEGATOR_OFFSET..DELEGATEE_OFFSET]
            .try_into()
            .map_err(|_| MultiDelegatorError::InvalidHeaderData)?;
        if delegator_bytes != accounts.authority.address().as_ref() {
            return Err(MultiDelegatorError::Unauthorized.into());
        }

        let payer_bytes: &[u8; 32] = data[PAYER_OFFSET..PAYER_OFFSET + 32]
            .try_into()
            .map_err(|_| MultiDelegatorError::InvalidPayerData)?;

        if payer_bytes == accounts.authority.address().as_ref() {
            accounts.authority
        } else {
            let receiver = accounts
                .receiver
                .ok_or(MultiDelegatorError::NotEnoughAccountKeys)?;
            if receiver.address().as_ref() != payer_bytes {
                return Err(MultiDelegatorError::Unauthorized.into());
            }
            receiver
        }
    };

    ProgramAccount::close(accounts.delegation_account, destination)
}

#[cfg(test)]
mod tests {
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;

    use crate::{
        tests::{
            asserts::TransactionResultExt,
            constants::{MINT_DECIMALS, TOKEN_PROGRAM_ID},
            utils::{
                current_ts, days, init_ata, init_mint, init_wallet,
                initialize_multidelegate_action, setup, CreateDelegation, RevokeDelegation,
            },
        },
        AccountDiscriminator, FixedDelegation, MultiDelegatorError, RecurringDelegation,
    };

    #[test]
    fn revoke_fixed_delegation() {
        let (litesvm, user) = &mut setup();
        let payer = user;

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(payer.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, payer.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, payer, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let (res, delegation_pda) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(nonce)
            .fixed(100, current_ts() + 1000);
        res.assert_ok();

        let account_before = litesvm.get_account(&delegation_pda);
        assert!(account_before.is_some());
        let binding = account_before.unwrap();
        let delegation_rent = binding.lamports;
        let delegation = FixedDelegation::load(&binding.data).unwrap();
        assert_eq!(
            delegation.header.discriminator,
            AccountDiscriminator::FixedDelegation as u8
        );

        let delegator_balance_before = litesvm.get_account(&payer.pubkey()).unwrap().lamports;

        let res = RevokeDelegation::new(litesvm, payer, mint, delegatee, nonce).execute();
        res.assert_ok();

        let account_after = litesvm.get_account(&delegation_pda);
        assert!(
            account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0
        );

        let delegator_balance_after = litesvm.get_account(&payer.pubkey()).unwrap().lamports;
        assert!(delegator_balance_after > delegator_balance_before);
        assert!(delegator_balance_after >= delegator_balance_before + delegation_rent - 10000);
    }

    #[test]
    fn revoke_recurring_delegation() {
        let (litesvm, user) = &mut setup();
        let payer = user;

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(payer.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, payer.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, payer, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let epoch = days(1);
        let expiry_ts = current_ts() + days(2) as i64;
        let (res, delegation_pda) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(nonce)
            .recurring(100, epoch, current_ts(), expiry_ts);
        res.assert_ok();

        let account_before = litesvm.get_account(&delegation_pda);
        assert!(account_before.is_some());
        let binding = account_before.unwrap();
        let delegation_rent = binding.lamports;
        let delegation = RecurringDelegation::load(&binding.data).unwrap();
        assert_eq!(
            delegation.header.discriminator,
            AccountDiscriminator::RecurringDelegation as u8
        );

        let delegator_balance_before = litesvm.get_account(&payer.pubkey()).unwrap().lamports;

        let res = RevokeDelegation::new(litesvm, payer, mint, delegatee, nonce).execute();
        res.assert_ok();

        let account_after = litesvm.get_account(&delegation_pda);
        assert!(
            account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0
        );

        let delegator_balance_after = litesvm.get_account(&payer.pubkey()).unwrap().lamports;
        assert!(delegator_balance_after > delegator_balance_before);
        assert!(delegator_balance_after >= delegator_balance_before + delegation_rent - 10000);
    }

    #[test]
    fn non_delegator_cannot_revoke() {
        let (litesvm, user) = &mut setup();
        let payer = user;

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(payer.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, payer.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, payer, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let epoch = days(1);
        let expiry_ts = current_ts() + days(2) as i64;
        let (res, delegation_pda) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(nonce)
            .recurring(100, epoch, current_ts(), expiry_ts);
        res.assert_ok();

        let attacker = init_wallet(litesvm, 1_000_000_000);
        let (multi_delegate_pda, _) =
            crate::tests::pda::get_multidelegate_pda(&payer.pubkey(), &mint);
        let res = revoke_delegation_action_with_pda(
            litesvm,
            &attacker,
            delegation_pda,
            multi_delegate_pda,
        );
        assert!(res.is_err());

        let account_after = litesvm.get_account(&delegation_pda);
        assert!(account_after.is_some());
        assert!(account_after.as_ref().map(|a| a.lamports).unwrap_or(0) > 0);
    }

    #[test]
    fn closed_account_is_zeroed() {
        let (litesvm, user) = &mut setup();
        let payer = user;

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(payer.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, payer.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, payer, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let (res, delegation_pda) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(nonce)
            .fixed(100, current_ts() + 1000);
        res.assert_ok();

        let account_before = litesvm.get_account(&delegation_pda);
        let _before_data = account_before.as_ref().unwrap().data.clone();

        let res = RevokeDelegation::new(litesvm, payer, mint, delegatee, nonce).execute();
        res.assert_ok();

        let account_after = litesvm.get_account(&delegation_pda);

        if let Some(account) = account_after {
            assert!(
                account.data.iter().all(|&byte| byte == 0),
                "All data should be zeroed after close"
            );
        }
    }

    #[test]
    fn revoke_with_wrong_receiver_returns_unauthorized() {
        let (litesvm, user) = &mut setup();
        let delegator = user;
        let sponsor = init_wallet(litesvm, 10_000_000_000);
        let wrong_receiver = init_wallet(litesvm, 10_000_000_000);

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(delegator.pubkey()),
            &[],
        );
        let _user_ata = init_ata(litesvm, mint, delegator.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, delegator, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let (res, _) = CreateDelegation::new(litesvm, delegator, mint, delegatee)
            .payer(&sponsor)
            .nonce(nonce)
            .fixed(100, current_ts() + 1000);
        res.assert_ok();

        let result = RevokeDelegation::new(litesvm, delegator, mint, delegatee, nonce)
            .receiver(wrong_receiver.pubkey())
            .execute();

        result.assert_err(MultiDelegatorError::Unauthorized);
    }

    #[allow(clippy::result_large_err)]
    fn revoke_delegation_action_with_pda(
        litesvm: &mut litesvm::LiteSVM,
        signer: &solana_keypair::Keypair,
        delegation_pda: Pubkey,
        _multi_delegate_pda: Pubkey,
    ) -> litesvm::types::TransactionResult {
        use solana_instruction::{AccountMeta, Instruction};
        use solana_signer::Signer;

        use crate::{
            instructions::revoke_delegation,
            tests::{constants::PROGRAM_ID, utils::build_and_send_transaction},
        };

        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(signer.pubkey(), true),
                AccountMeta::new(delegation_pda, false),
            ],
            data: vec![*revoke_delegation::DISCRIMINATOR],
        };

        build_and_send_transaction(litesvm, &[signer], &signer.pubkey(), &ix)
    }
}
