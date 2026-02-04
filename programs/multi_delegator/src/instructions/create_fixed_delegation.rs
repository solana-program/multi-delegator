use core::mem::{size_of, transmute};
use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, ProgramResult};
use shank::ShankType;

use crate::{
    create_delegation_account, init_header, state::FixedDelegation, CreateDelegationAccounts,
    DelegationKind, MultiDelegatorError,
};

#[repr(C, packed)]
#[derive(Debug, ShankType)]
pub struct CreateFixedDelegationData {
    pub nonce: u64,
    pub amount: u64,
    pub expiry_ts: i64,
}

impl CreateFixedDelegationData {
    pub const LEN: usize = size_of::<CreateFixedDelegationData>();

    fn load(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() != Self::LEN {
            msg!(&format!(
                "Data.len() = {}. Expected = {}",
                data.len(),
                Self::LEN
            ));
            return Err(MultiDelegatorError::InvalidInstructionData.into());
        }
        Ok(unsafe { &*transmute::<*const u8, *const Self>(data.as_ptr()) })
    }
}

pub const DISCRIMINATOR: &u8 = &1;

pub fn process((data, accounts): (&[u8], &[AccountInfo])) -> ProgramResult {
    let accounts = CreateDelegationAccounts::try_from(accounts)?;
    let call_data = CreateFixedDelegationData::load(data)?;

    let bump = create_delegation_account(&accounts, call_data.nonce, FixedDelegation::LEN)?;

    let binding = &mut accounts.delegation_account.try_borrow_mut_data()?;
    let delegation = FixedDelegation::load_mut(binding)?;

    init_header(
        &mut delegation.header,
        DelegationKind::Fixed,
        bump,
        accounts.delegator.key(),
        accounts.delegatee.key(),
    );
    delegation.amount = call_data.amount;
    delegation.expiry_ts = call_data.expiry_ts;

    Ok(())
}

#[cfg(test)]
mod tests {
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;

    use crate::{
        tests::{
            constants::{MINT_DECIMALS, TOKEN_PROGRAM_ID},
            pda::get_delegation_pda,
            utils::{
                create_fixed_delegation_action, create_fixed_delegation_action_with_pda,
                current_ts, days, init_ata, init_mint, initialize_multidelegate_action, setup,
            },
        },
        DelegationKind, FixedDelegation,
    };

    #[test]
    fn create_fixed_delegation() {
        let (litesvm, user) = &mut setup();
        let payer = user;
        let amount: u64 = 100_000_000;
        let expiry_ts: i64 = current_ts() + days(1) as i64;
        let nonce: u64 = 0;

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

        let (res, delegation_pda) = create_fixed_delegation_action(
            litesvm, payer, mint, delegatee, nonce, amount, expiry_ts,
        );
        res.unwrap();

        let account = litesvm.get_account(&delegation_pda).unwrap();
        let delegation = FixedDelegation::load(&account.data).unwrap();

        let header = delegation.header;
        let del_amount = delegation.amount;
        let del_expiry_s = delegation.expiry_ts;
        assert_eq!(header.delegator, payer.pubkey().to_bytes());
        assert_eq!(header.delegatee, delegatee.to_bytes());
        assert_eq!(header.kind, DelegationKind::Fixed as u8);
        assert_eq!(del_amount, amount);
        assert_eq!(del_expiry_s, expiry_ts);
    }

    // NOTE: These error tests use FixedDelegation but validate shared code paths.
    // The same checks apply to RecurringDelegation via shared helpers.

    #[test]
    fn create_delegation_without_multidelegate() {
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

        let delegatee = Pubkey::new_unique();
        let (res, _) =
            create_fixed_delegation_action(litesvm, payer, mint, delegatee, 0, 100, 1000);

        assert!(res.is_err());
    }

    #[test]
    fn create_delegation_wrong_pda() {
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
        let wrong_pda = Pubkey::new_unique();

        let res = create_fixed_delegation_action_with_pda(
            litesvm, payer, mint, delegatee, wrong_pda, 0, 100, 1000,
        );

        assert!(res.is_err());
    }

    #[test]
    fn create_delegation_duplicate_nonce() {
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

        let (res, _) =
            create_fixed_delegation_action(litesvm, payer, mint, delegatee, 0, 100, 1000);
        res.unwrap();

        let (res2, _) =
            create_fixed_delegation_action(litesvm, payer, mint, delegatee, 0, 200, 2000);
        assert!(res2.is_err());
    }

    #[test]
    fn create_multiple_delegations_different_nonces() {
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

        let (_, multi_delegate_pda, _) = initialize_multidelegate_action(litesvm, payer, mint);

        let delegatee = Pubkey::new_unique();

        let (res0, pda0) =
            create_fixed_delegation_action(litesvm, payer, mint, delegatee, 0, 100, 1000);
        res0.unwrap();

        let (res1, pda1) =
            create_fixed_delegation_action(litesvm, payer, mint, delegatee, 1, 200, 2000);
        res1.unwrap();

        let (res2, pda2) =
            create_fixed_delegation_action(litesvm, payer, mint, delegatee, 2, 300, 3000);
        res2.unwrap();

        assert_ne!(pda0, pda1);
        assert_ne!(pda1, pda2);
        assert_ne!(pda0, pda2);

        let (expected_pda0, _) =
            get_delegation_pda(&multi_delegate_pda, &payer.pubkey(), &delegatee, 0);
        let (expected_pda1, _) =
            get_delegation_pda(&multi_delegate_pda, &payer.pubkey(), &delegatee, 1);
        let (expected_pda2, _) =
            get_delegation_pda(&multi_delegate_pda, &payer.pubkey(), &delegatee, 2);

        assert_eq!(pda0, expected_pda0);
        assert_eq!(pda1, expected_pda1);
        assert_eq!(pda2, expected_pda2);
    }
}
