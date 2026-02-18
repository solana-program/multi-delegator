use codama::CodamaAccount;
use core::mem::{size_of, transmute};
use pinocchio::error::ProgramError;

use crate::{
    state::common::AccountDiscriminator, state::header::DISCRIMINATOR_OFFSET, Header,
    MultiDelegatorError,
};

#[repr(C, packed)]
#[derive(Debug, CodamaAccount)]
pub struct RecurringDelegation {
    pub header: Header,
    /// Timestamp start of the current period
    pub current_period_start_ts: i64,
    /// Length of a period in seconds
    pub period_length_s: u64,
    /// Expirey after which this delegation will no longer be active
    pub expiry_ts: i64,
    /// How much can be transfered each period
    pub amount_per_period: u64,
    /// How much has been transfered so far in this period
    pub amount_pulled_in_period: u64,
}

impl RecurringDelegation {
    pub const LEN: usize = size_of::<Self>();

    pub fn load(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        if bytes[DISCRIMINATOR_OFFSET] != AccountDiscriminator::RecurringDelegation as u8 {
            return Err(MultiDelegatorError::InvalidAccountDiscriminator.into());
        }
        Ok(unsafe { &*transmute::<*const u8, *const Self>(bytes.as_ptr()) })
    }

    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        if bytes[DISCRIMINATOR_OFFSET] != AccountDiscriminator::RecurringDelegation as u8 {
            return Err(MultiDelegatorError::InvalidAccountDiscriminator.into());
        }
        Ok(unsafe { &mut *transmute::<*mut u8, *mut Self>(bytes.as_mut_ptr()) })
    }
}
