use core::mem::transmute;
use pinocchio::program_error::ProgramError;
use shank::ShankAccount;

use super::header::Header;
use crate::MultiDelegatorError;

#[repr(C, packed)]
#[derive(Debug, ShankAccount)]
pub struct RecurringDelegation {
    pub header: Header,
    pub last_pull_ts: i64,
    pub period_length_s: u64,
    pub expiry_s: u64,
    pub amount_per_period: u64,
    pub amount_pulled_in_period: u64,
}

impl RecurringDelegation {
    pub const LEN: usize = core::mem::size_of::<RecurringDelegation>();

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
