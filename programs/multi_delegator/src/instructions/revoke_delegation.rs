use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

use crate::{
    AccountCheck, AccountClose, Header, MultiDelegatorError, ProgramAccount, SignerAccount,
    DELEGATEE_OFFSET, DELEGATOR_OFFSET,
};

pub struct RevokeDelegationAccounts<'a> {
    pub authority: &'a AccountInfo,
    pub delegation_account: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for RevokeDelegationAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, delegation_account, ..] = accounts else {
            return Err(MultiDelegatorError::NotEnoughAccountKeys.into());
        };

        SignerAccount::check(authority)?;
        ProgramAccount::check(delegation_account)?;

        Ok(Self {
            authority,
            delegation_account,
        })
    }
}

pub const DISCRIMINATOR: &u8 = &3;

/// Revokes a delegation by closing the delegation PDA.
/// The rent lamports are always returned to the delegator (authority).
pub fn process(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts = RevokeDelegationAccounts::try_from(accounts)?;

    {
        let data = accounts.delegation_account.try_borrow_data()?;

        if data.len() < Header::LEN {
            return Err(MultiDelegatorError::InvalidHeaderData.into());
        }

        let delegator_bytes: &[u8; 32] = data[DELEGATOR_OFFSET..DELEGATEE_OFFSET]
            .try_into()
            .map_err(|_| MultiDelegatorError::InvalidHeaderData)?;
        if delegator_bytes != accounts.authority.key().as_ref() {
            return Err(MultiDelegatorError::Unauthorized.into());
        }
    }

    ProgramAccount::close(accounts.delegation_account, accounts.authority)
}

#[cfg(test)]
mod tests {
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;

    use crate::{
        tests::{
            constants::{MINT_DECIMALS, TOKEN_PROGRAM_ID},
            utils::{
                create_fixed_delegation_action, create_recurring_delegation_action, current_ts,
                days, init_ata, init_mint, init_wallet, initialize_multidelegate_action,
                revoke_delegation_action, setup,
            },
        },
        DelegationKind, FixedDelegation, RecurringDelegation,
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
        );
        let _user_ata = init_ata(litesvm, mint, payer.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, payer, mint)
            .0
            .unwrap();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let (res, delegation_pda) =
            create_fixed_delegation_action(litesvm, payer, mint, delegatee, nonce, 100, 1000);
        res.unwrap();

        let account_before = litesvm.get_account(&delegation_pda);
        assert!(account_before.is_some());
        let binding = account_before.unwrap();
        let delegation_rent = binding.lamports;
        let delegation = FixedDelegation::load(&binding.data).unwrap();
        assert_eq!(delegation.header.kind, DelegationKind::Fixed as u8);

        let delegator_balance_before = litesvm.get_account(&payer.pubkey()).unwrap().lamports;

        let res = revoke_delegation_action(litesvm, payer, mint, delegatee, nonce);
        res.unwrap();

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
        );
        let _user_ata = init_ata(litesvm, mint, payer.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, payer, mint)
            .0
            .unwrap();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let epoch = days(1);
        let expiry_ts = current_ts() + days(2) as i64;
        let (res, delegation_pda) = create_recurring_delegation_action(
            litesvm,
            payer,
            mint,
            delegatee,
            nonce,
            100,
            epoch,
            current_ts(),
            expiry_ts,
        );
        res.unwrap();

        let account_before = litesvm.get_account(&delegation_pda);
        assert!(account_before.is_some());
        let binding = account_before.unwrap();
        let delegation_rent = binding.lamports;
        let delegation = RecurringDelegation::load(&binding.data).unwrap();
        assert_eq!(delegation.header.kind, DelegationKind::Recurring as u8);

        let delegator_balance_before = litesvm.get_account(&payer.pubkey()).unwrap().lamports;

        let res = revoke_delegation_action(litesvm, payer, mint, delegatee, nonce);
        res.unwrap();

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
        );
        let _user_ata = init_ata(litesvm, mint, payer.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, payer, mint)
            .0
            .unwrap();

        let delegatee = Pubkey::new_unique();
        let nonce: u64 = 0;

        let epoch = days(1);
        let expiry_ts = current_ts() + days(2) as i64;
        let (res, delegation_pda) = create_recurring_delegation_action(
            litesvm,
            payer,
            mint,
            delegatee,
            nonce,
            100,
            epoch,
            current_ts(),
            expiry_ts,
        );
        res.unwrap();

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
