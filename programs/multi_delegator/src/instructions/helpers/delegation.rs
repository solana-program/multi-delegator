use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
};
use pinocchio_system::instructions::CreateAccount;

use crate::{
    find_delegation_pda, AccountCheck, DelegationKind, Header, MultiDelegateAccount,
    MultiDelegatorError, SignerAccount, SystemAccount, CURRENT_VERSION, DELEGATE_BASE_SEED,
};

pub struct CreateDelegationAccounts<'a> {
    pub delegator: &'a AccountInfo,
    pub multi_delegate: &'a AccountInfo,
    pub delegation_account: &'a AccountInfo,
    pub delegatee: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for CreateDelegationAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [delegator, multi_delegate, delegation_account, delegatee, system_program, ..] =
            accounts
        else {
            return Err(MultiDelegatorError::NotEnoughAccountKeys.into());
        };

        SignerAccount::check(delegator)?;
        SystemAccount::check(system_program)?;
        MultiDelegateAccount::check(multi_delegate)?;

        Ok(Self {
            delegator,
            multi_delegate,
            delegation_account,
            delegatee,
            system_program,
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
        from: accounts.delegator,
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
) {
    header.version = CURRENT_VERSION;
    header.kind = kind.into();
    header.bump = bump;
    header.delegator = *delegator;
    header.delegatee = *delegatee;
}
