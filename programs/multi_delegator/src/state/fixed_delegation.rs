use core::mem::transmute;
use pinocchio::program_error::ProgramError;
use shank::ShankAccount;

use super::header::Header;
use crate::MultiDelegatorError;

#[repr(C, packed)]
#[derive(Debug, ShankAccount)]
pub struct FixedDelegation {
    pub header: Header,
    pub amount: u64,
    pub expiry_s: u64,
}

impl FixedDelegation {
    pub const LEN: usize = core::mem::size_of::<FixedDelegation>();

    #[inline(always)]
    pub fn load(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        Ok(unsafe { &*transmute::<*const u8, *const Self>(bytes.as_ptr()) })
    }

    #[inline(always)]
    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        Ok(unsafe { &mut *transmute::<*mut u8, *mut Self>(bytes.as_mut_ptr()) })
    }
}
