use super::traits::AccountCheck;
use crate::MultiDelegatorError;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

pub struct SignerAccount;

impl AccountCheck for SignerAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_signer() {
            return Err(MultiDelegatorError::NotSigner.into());
        }
        Ok(())
    }
}

pub struct SystemAccount;

impl AccountCheck for SystemAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if account.key().ne(&pinocchio_system::ID) {
            return Err(MultiDelegatorError::NotSystemProgram.into());
        }

        Ok(())
    }
}
