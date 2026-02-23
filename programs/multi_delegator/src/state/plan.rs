use codama::CodamaAccount;
use core::mem::{size_of, transmute};
use pinocchio::{error::ProgramError, Address};

use crate::{state::common::AccountDiscriminator, MultiDelegatorError};

pub use crate::instructions::create_plan::PlanData;

pub const PLAN_DISCRIMINATOR_OFFSET: usize = 0;

#[repr(C, packed)]
#[derive(CodamaAccount)]
pub struct Plan {
    pub discriminator: u8,
    pub owner: Address,
    pub bump: u8,
    pub status: u8,
    pub data: PlanData,
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

    /// Check that the caller is authorized to pull from this plan.
    /// The caller must be the plan owner or listed in the pullers array.
    pub fn can_pull(&self, caller: &Address) -> Result<(), ProgramError> {
        if *caller == self.owner {
            return Ok(());
        }
        if self.data.pullers.contains(caller) {
            return Ok(());
        }
        Err(MultiDelegatorError::Unauthorized.into())
    }

    /// Validate that the receiver owner is an allowed destination.
    /// If no destinations are configured (all zero), any receiver is valid.
    pub fn check_destination(&self, receiver_owner: &Address) -> Result<(), ProgramError> {
        let zero = Address::default();
        let has_destinations = self.data.destinations.iter().any(|d| *d != zero);
        if has_destinations && !self.data.destinations.contains(receiver_owner) {
            return Err(MultiDelegatorError::UnauthorizedDestination.into());
        }
        Ok(())
    }
}
