use codama::CodamaType;
use core::mem::{size_of, transmute};
use pinocchio::{error::ProgramError, AccountView, ProgramResult};

use crate::{
    create_delegation_account, init_header, state::FixedDelegation, AccountDiscriminator,
    CreateDelegationAccounts, MultiDelegatorError, DISCRIMINATOR_OFFSET,
};

#[repr(C, packed)]
#[derive(Debug, Clone, CodamaType)]
pub struct CreateFixedDelegationData {
    pub nonce: u64,
    pub amount: u64,
    pub expiry_ts: i64,
}

impl CreateFixedDelegationData {
    pub const LEN: usize = size_of::<CreateFixedDelegationData>();

    pub fn load(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidInstructionData.into());
        }
        Ok(unsafe { &*transmute::<*const u8, *const Self>(data.as_ptr()) })
    }
}

pub const DISCRIMINATOR: &u8 = &1;

pub fn process(accounts: &[AccountView], call_data: &CreateFixedDelegationData) -> ProgramResult {
    let accounts = CreateDelegationAccounts::try_from(accounts)?;

    let bump = create_delegation_account(&accounts, call_data.nonce, FixedDelegation::LEN)?;

    let binding = &mut accounts.delegation_account.try_borrow_mut()?;
    // Set discriminator before load_mut so validation passes on freshly created account
    binding[DISCRIMINATOR_OFFSET] = AccountDiscriminator::FixedDelegation as u8;
    let delegation = FixedDelegation::load_mut(binding)?;

    init_header(
        &mut delegation.header,
        AccountDiscriminator::FixedDelegation,
        bump,
        accounts.delegator.address(),
        accounts.delegatee.address(),
        accounts.payer.address(),
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
            asserts::TransactionResultExt,
            constants::{MINT_DECIMALS, TOKEN_PROGRAM_ID},
            pda::get_delegation_pda,
            utils::{
                current_ts, days, init_ata, init_mint, init_wallet,
                initialize_multidelegate_action, setup, CreateDelegation, RevokeDelegation,
            },
        },
        AccountDiscriminator, FixedDelegation,
    };

    #[test]
    fn create_fixed_delegation_with_sponsor() {
        let (litesvm, user) = &mut setup();
        let delegator = user;
        let sponsor = init_wallet(litesvm, 10_000_000_000);

        let amount: u64 = 100_000_000;
        let expiry_ts: i64 = current_ts() + days(1) as i64;
        let nonce: u64 = 0;

        let mint = init_mint(
            litesvm,
            TOKEN_PROGRAM_ID,
            MINT_DECIMALS,
            1_000_000_000,
            Some(delegator.pubkey()),
        );
        let _user_ata = init_ata(litesvm, mint, delegator.pubkey(), 1_000_000);

        initialize_multidelegate_action(litesvm, delegator, mint)
            .0
            .assert_ok();

        let delegatee = Pubkey::new_unique();

        let delegator_balance_before = litesvm.get_account(&delegator.pubkey()).unwrap().lamports;
        let sponsor_balance_before = litesvm.get_account(&sponsor.pubkey()).unwrap().lamports;

        let (res, delegation_pda) = CreateDelegation::new(litesvm, delegator, mint, delegatee)
            .payer(&sponsor)
            .nonce(nonce)
            .fixed(amount, expiry_ts);
        res.assert_ok();

        let delegator_balance_after = litesvm.get_account(&delegator.pubkey()).unwrap().lamports;
        let sponsor_balance_after = litesvm.get_account(&sponsor.pubkey()).unwrap().lamports;

        assert_eq!(delegator_balance_after, delegator_balance_before); // Delegator shouldn't spend anything
        assert!(sponsor_balance_after < sponsor_balance_before);

        let account = litesvm.get_account(&delegation_pda).unwrap();
        let delegation_rent = account.lamports;
        let delegation = FixedDelegation::load(&account.data).unwrap();

        assert_eq!(
            delegation.header.payer.to_bytes(),
            sponsor.pubkey().to_bytes()
        );

        // Now revoke and check refund
        let res = RevokeDelegation::new(litesvm, delegator, mint, delegatee, nonce)
            .receiver(sponsor.pubkey())
            .execute();
        res.assert_ok();

        let sponsor_balance_final = litesvm.get_account(&sponsor.pubkey()).unwrap().lamports;

        assert!(sponsor_balance_final >= sponsor_balance_after + delegation_rent);

        // Check delegator paid for revoke
        let delegator_balance_final = litesvm.get_account(&delegator.pubkey()).unwrap().lamports;
        assert!(delegator_balance_final < delegator_balance_after);
    }

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
            .assert_ok();

        let delegatee = Pubkey::new_unique();

        let (res, delegation_pda) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(nonce)
            .fixed(amount, expiry_ts);
        res.assert_ok();

        let account = litesvm.get_account(&delegation_pda).unwrap();
        let delegation = FixedDelegation::load(&account.data).unwrap();

        let header = delegation.header;
        let del_amount = delegation.amount;
        let del_expiry_s = delegation.expiry_ts;
        assert_eq!(header.delegator.to_bytes(), payer.pubkey().to_bytes());
        assert_eq!(header.delegatee.to_bytes(), delegatee.to_bytes());
        assert_eq!(
            header.discriminator,
            AccountDiscriminator::FixedDelegation as u8
        );
        assert_eq!(del_amount, amount);
        assert_eq!(del_expiry_s, expiry_ts);
    }

    /// Verify that pre-funding a delegation PDA with lamports (DOS attack)
    /// does not prevent the legitimate user from creating the delegation.
    #[test]
    fn create_fixed_delegation_with_prefunded_pda() {
        use solana_account::Account;

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
            .assert_ok();

        let delegatee = solana_pubkey::Pubkey::new_unique();

        // Simulate an attacker pre-funding the delegation PDA address with lamports
        let (multi_delegate_pda, _) = get_delegation_pda(
            &crate::tests::pda::get_multidelegate_pda(&payer.pubkey(), &mint).0,
            &payer.pubkey(),
            &delegatee,
            nonce,
        );
        litesvm
            .set_account(
                multi_delegate_pda,
                Account {
                    lamports: 1_000,
                    data: vec![],
                    owner: solana_pubkey::Pubkey::default(), // system program
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

        // The user should still be able to create the delegation PDA
        let (res, delegation_pda) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(nonce)
            .fixed(amount, expiry_ts);
        res.assert_ok();

        let account = litesvm.get_account(&delegation_pda).unwrap();
        let delegation = FixedDelegation::load(&account.data).unwrap();

        let header = delegation.header;
        let del_amount = delegation.amount;
        let del_expiry_ts = delegation.expiry_ts;
        assert_eq!(header.delegator.to_bytes(), payer.pubkey().to_bytes());
        assert_eq!(header.delegatee.to_bytes(), delegatee.to_bytes());
        assert_eq!(
            header.discriminator,
            AccountDiscriminator::FixedDelegation as u8
        );
        assert_eq!(del_amount, amount);
        assert_eq!(del_expiry_ts, expiry_ts);
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
        let (res, _) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(0)
            .fixed(100, 1000);

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
            .assert_ok();

        let delegatee = Pubkey::new_unique();
        let wrong_pda = Pubkey::new_unique();

        let (res, _) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .pda(wrong_pda)
            .nonce(0)
            .fixed(100, 1000);

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
            .assert_ok();

        let delegatee = Pubkey::new_unique();

        let (res, _) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(0)
            .fixed(100, 1000);
        res.assert_ok();

        let (res2, _) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(0)
            .fixed(200, 2000);
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

        let (res0, pda0) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(0)
            .fixed(100, 1000);
        let tx = res0.assert_ok();
        println!(
            "Create Fixed delegation consumed: {} CUs",
            tx.compute_units_consumed
        );

        let (res1, pda1) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(1)
            .fixed(200, 2000);
        let tx = res1.assert_ok();
        println!(
            "Create Fixed delegation consumed: {} CUs",
            tx.compute_units_consumed
        );

        let (res2, pda2) = CreateDelegation::new(litesvm, payer, mint, delegatee)
            .nonce(2)
            .fixed(300, 3000);
        let tx = res2.assert_ok();
        println!(
            "Create Fixed delegation consumed: {} CUs",
            tx.compute_units_consumed
        );

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
