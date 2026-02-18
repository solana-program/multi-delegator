use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address,
};
use pinocchio_system::instructions::CreateAccount;

use crate::{
    state::common::find_delegation_pda, AccountCheck, AccountDiscriminator, Header,
    MultiDelegateAccount, MultiDelegatorError, SignerAccount, SystemAccount, CURRENT_VERSION,
    DELEGATE_BASE_SEED,
};

pub struct CreateDelegationAccounts<'a> {
    pub delegator: &'a AccountView,
    pub multi_delegate: &'a AccountView,
    pub delegation_account: &'a AccountView,
    pub delegatee: &'a AccountView,
    pub system_program: &'a AccountView,
    pub payer: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for CreateDelegationAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [delegator, multi_delegate, delegation_account, delegatee, system_program, rem @ ..] =
            accounts
        else {
            return Err(MultiDelegatorError::NotEnoughAccountKeys.into());
        };

        SignerAccount::check(delegator)?;
        SystemAccount::check(system_program)?;
        MultiDelegateAccount::check(multi_delegate)?;

        let payer = if let Some(payer) = rem.first() {
            SignerAccount::check(payer)?;
            payer
        } else {
            delegator
        };

        Ok(Self {
            delegator,
            multi_delegate,
            delegation_account,
            delegatee,
            system_program,
            payer,
        })
    }
}

pub fn create_delegation_account(
    accounts: &CreateDelegationAccounts,
    nonce: u64,
    space: usize,
) -> Result<u8, ProgramError> {
    let nonce_bytes = nonce.to_le_bytes();

    let (expected_pda, bump) = find_delegation_pda(
        accounts.multi_delegate.address(),
        accounts.delegator.address(),
        accounts.delegatee.address(),
        nonce,
    );

    if expected_pda != *accounts.delegation_account.address() {
        return Err(MultiDelegatorError::InvalidDelegatePda.into());
    }

    let lamports = Rent::get()?.try_minimum_balance(space)?;
    let bump_bytes = [bump];
    let seeds = [
        Seed::from(DELEGATE_BASE_SEED),
        Seed::from(accounts.multi_delegate.address().as_ref()),
        Seed::from(accounts.delegator.address().as_ref()),
        Seed::from(accounts.delegatee.address().as_ref()),
        Seed::from(&nonce_bytes),
        Seed::from(&bump_bytes),
    ];
    let signer = [Signer::from(&seeds)];

    CreateAccount {
        from: accounts.payer,
        to: accounts.delegation_account,
        lamports,
        space: space as u64,
        owner: &crate::ID,
    }
    .invoke_signed(&signer)?;

    Ok(bump)
}

pub fn init_header(
    header: &mut Header,
    discriminator: AccountDiscriminator,
    bump: u8,
    delegator: &Address,
    delegatee: &Address,
    payer: &Address,
) {
    header.version = CURRENT_VERSION;
    header.discriminator = discriminator.into();
    header.bump = bump;
    header.delegator = *delegator;
    header.delegatee = *delegatee;
    header.payer = *payer;
}

/// Authorization checker for delegation transfers.
///
/// Verifies that the delegation belongs to the claimed delegator and that
/// the caller is the authorized delegatee. This prevents an attacker from
/// using their own delegation to transfer funds from another user's account.
pub struct Delegation;

impl Delegation {
    /// Checks that:
    /// 1. The delegation belongs to the claimed delegator
    /// 2. The caller is the authorized delegatee for this delegation
    pub fn check(
        header: &Header,
        expected_delegator: &Address,
        caller_delegatee: &Address,
    ) -> Result<(), ProgramError> {
        if header.delegator != *expected_delegator {
            return Err(MultiDelegatorError::Unauthorized.into());
        }
        if header.delegatee != *caller_delegatee {
            return Err(MultiDelegatorError::Unauthorized.into());
        }
        Ok(())
    }
}
