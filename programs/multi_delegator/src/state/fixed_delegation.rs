use codama::CodamaAccount;
use core::mem::{size_of, transmute};
use pinocchio::error::ProgramError;

use crate::{Header, MultiDelegatorError};

#[repr(C, packed)]
#[derive(Debug, CodamaAccount)]
pub struct FixedDelegation {
    pub header: Header,
    pub amount: u64,
    pub expiry_ts: i64,
}

impl FixedDelegation {
    pub const LEN: usize = size_of::<Self>();

    pub fn load(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        Ok(unsafe { &*transmute::<*const u8, *const Self>(bytes.as_ptr()) })
    }

    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        Ok(unsafe { &mut *transmute::<*mut u8, *mut Self>(bytes.as_mut_ptr()) })
    }
}
