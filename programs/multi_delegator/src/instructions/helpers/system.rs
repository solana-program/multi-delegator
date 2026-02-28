use super::traits::AccountCheck;
use crate::{
    state::common::AccountDiscriminator, MultiDelegate, MultiDelegatorError, DISCRIMINATOR_OFFSET,
};
use pinocchio::{error::ProgramError, AccountView};

pub struct SignerAccount;

impl AccountCheck for SignerAccount {
    fn check(account: &AccountView) -> Result<(), ProgramError> {
        if !account.is_signer() {
            return Err(MultiDelegatorError::NotSigner.into());
        }
        Ok(())
    }
}

pub struct WritableAccount;

impl AccountCheck for WritableAccount {
    fn check(account: &AccountView) -> Result<(), ProgramError> {
        if !account.is_writable() {
            return Err(MultiDelegatorError::AccountNotWritable.into());
        }
        Ok(())
    }
}

pub struct SystemAccount;

impl AccountCheck for SystemAccount {
    fn check(account: &AccountView) -> Result<(), ProgramError> {
        if account.address().ne(&pinocchio_system::ID) {
            return Err(MultiDelegatorError::NotSystemProgram.into());
        }

        Ok(())
    }
}

pub struct MultiDelegateAccount;

impl AccountCheck for MultiDelegateAccount {
    fn check(account: &AccountView) -> Result<(), ProgramError> {
        if !account.owned_by(&crate::ID) {
            return Err(MultiDelegatorError::InvalidMultiDelegatePda.into());
        }
        let data = account.try_borrow()?;
        if data.len() != MultiDelegate::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        if data[DISCRIMINATOR_OFFSET] != AccountDiscriminator::MultiDelegate as u8 {
            return Err(MultiDelegatorError::InvalidAccountDiscriminator.into());
        }
        Ok(())
    }
}
