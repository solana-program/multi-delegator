use codama::CodamaAccount;
use core::mem::{size_of, transmute};
use pinocchio::{error::ProgramError, Address};

use crate::{state::common::AccountDiscriminator, MultiDelegatorError};

pub const PLAN_DISCRIMINATOR_OFFSET: usize = 0;

#[repr(C, packed)]
#[derive(Debug, CodamaAccount)]
pub struct Plan {
    pub discriminator: u8,
    pub owner: Address,
    pub bump: u8,
    pub status: u8,
    pub plan_id: u64,
    pub mint: Address,
    pub amount: u64,
    pub period_hours: u64,
    pub end_ts: i64,
    pub destinations: [Address; 4],
    pub pullers: [Address; 4],
    pub metadata_uri: [u8; 128],
}

impl Plan {
    pub const LEN: usize = size_of::<Self>();
    pub const SEED: &'static [u8] = b"plan";

    pub fn load(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        if bytes[PLAN_DISCRIMINATOR_OFFSET] != AccountDiscriminator::Plan as u8 {
            return Err(MultiDelegatorError::InvalidAccountDiscriminator.into());
        }
        Ok(unsafe { &*transmute::<*const u8, *const Self>(bytes.as_ptr()) })
    }

    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        if bytes[PLAN_DISCRIMINATOR_OFFSET] != AccountDiscriminator::Plan as u8 {
            return Err(MultiDelegatorError::InvalidAccountDiscriminator.into());
        }
        Ok(unsafe { &mut *transmute::<*mut u8, *mut Self>(bytes.as_mut_ptr()) })
    }
}
