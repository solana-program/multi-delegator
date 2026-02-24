use codama::CodamaAccount;
use core::mem::{size_of, transmute};
use pinocchio::error::ProgramError;

use crate::{
    state::common::AccountDiscriminator, state::header::DISCRIMINATOR_OFFSET, Header,
    MultiDelegatorError,
};

#[repr(C, packed)]
#[derive(Debug, CodamaAccount)]
pub struct SubscriptionDelegation {
    pub header: Header,
    pub amount_pulled_in_period: u64,
    pub current_period_start_ts: i64,
    pub expires_at_ts: i64,
}

impl SubscriptionDelegation {
    pub const LEN: usize = size_of::<Self>();
    pub const SEED: &'static [u8] = b"subscription";

    pub fn load(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        if bytes[DISCRIMINATOR_OFFSET] != AccountDiscriminator::SubscriptionDelegation as u8 {
            return Err(MultiDelegatorError::InvalidAccountDiscriminator.into());
        }
        Ok(unsafe { &*transmute::<*const u8, *const Self>(bytes.as_ptr()) })
    }

    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        if bytes[DISCRIMINATOR_OFFSET] != AccountDiscriminator::SubscriptionDelegation as u8 {
            return Err(MultiDelegatorError::InvalidAccountDiscriminator.into());
        }
        Ok(unsafe { &mut *transmute::<*mut u8, *mut Self>(bytes.as_mut_ptr()) })
    }
}
