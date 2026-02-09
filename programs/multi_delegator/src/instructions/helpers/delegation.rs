use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
};
use pinocchio_system::instructions::CreateAccount;

use crate::{
    state::common::find_delegation_pda, AccountCheck, DelegationKind, Header, MultiDelegateAccount,
    MultiDelegatorError, SignerAccount, SystemAccount, CURRENT_VERSION, DELEGATE_BASE_SEED,
};

pub struct CreateDelegationAccounts<'a> {
    pub delegator: &'a AccountInfo,
    pub multi_delegate: &'a AccountInfo,
    pub delegation_account: &'a AccountInfo,
    pub delegatee: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
    pub payer: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for CreateDelegationAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
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
        accounts.multi_delegate.key(),
        accounts.delegator.key(),
        accounts.delegatee.key(),
        nonce,
    );

    if expected_pda != *accounts.delegation_account.key() {
        return Err(MultiDelegatorError::InvalidDelegatePda.into());
    }

    let lamports = Rent::get()?.minimum_balance(space);
    let bump_bytes = [bump];
    let seeds = [
        Seed::from(DELEGATE_BASE_SEED),
        Seed::from(accounts.multi_delegate.key().as_ref()),
        Seed::from(accounts.delegator.key().as_ref()),
        Seed::from(accounts.delegatee.key().as_ref()),
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
    kind: DelegationKind,
    bump: u8,
    delegator: &Pubkey,
    delegatee: &Pubkey,
    payer: &Pubkey,
) {
    header.version = CURRENT_VERSION;
    header.kind = kind.into();
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
        expected_delegator: &Pubkey,
        caller_delegatee: &Pubkey,
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
