use codama::CodamaAccount;
use core::mem::{size_of, transmute};
use pinocchio::{error::ProgramError, Address};

use crate::MultiDelegatorError;

#[repr(C)]
#[derive(CodamaAccount)]
pub struct MultiDelegate {
    pub user: Address,
    pub token_mint: Address,
    pub bump: u8,
}

impl MultiDelegate {
    pub const LEN: usize = size_of::<Address>() + size_of::<Address>() + size_of::<u8>();
    pub const SEED: &'static [u8] = b"MultiDelegate";

    #[inline(always)]
    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        Ok(unsafe { &mut *transmute::<*mut u8, *mut Self>(bytes.as_mut_ptr()) })
    }

    #[inline(always)]
    pub fn load(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        Ok(unsafe { &*transmute::<*const u8, *const Self>(bytes.as_ptr()) })
    }

    /// Verifies that the given seeds and bump produce a valid PDA.
    /// This is cheaper than find_pda as it doesn't iterate through bumps.
    /// Returns the computed PDA if valid, or an error if the bump is invalid.
    pub fn verify_pda(
        user: &Address,
        token_mint: &Address,
        bump: u8,
    ) -> Result<Address, ProgramError> {
        Address::create_program_address(
            &[Self::SEED, user.as_ref(), token_mint.as_ref(), &[bump]],
            &crate::ID,
        )
        .map_err(|_| MultiDelegatorError::InvalidMultiDelegatePda.into())
    }

    /// Finds the canonical PDA and bump for the multi-delegate account.
    /// Used when creating the multi-delegate to ensure the canonical bump is used.
    pub fn find_pda(user: &Address, token_mint: &Address) -> (Address, u8) {
        Address::find_program_address(
            &[Self::SEED, user.as_ref(), token_mint.as_ref()],
            &crate::ID,
        )
    }
}
