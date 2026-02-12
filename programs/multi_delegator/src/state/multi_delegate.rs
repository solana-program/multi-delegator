use codama::CodamaAccount;
use core::mem::{size_of, transmute};
use pinocchio::{
    program_error::ProgramError,
    pubkey::{create_program_address, find_program_address, Pubkey},
};

use crate::MultiDelegatorError;

#[repr(C)]
#[derive(CodamaAccount)]
pub struct MultiDelegate {
    pub user: Pubkey,
    pub token_mint: Pubkey,
    pub bump: u8,
}

impl MultiDelegate {
    pub const LEN: usize = size_of::<Pubkey>() + size_of::<Pubkey>() + size_of::<u8>();
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
        user: &Pubkey,
        token_mint: &Pubkey,
        bump: u8,
    ) -> Result<Pubkey, ProgramError> {
        create_program_address(
            &[Self::SEED, user.as_ref(), token_mint.as_ref(), &[bump]],
            &crate::ID,
        )
        .map_err(|_| MultiDelegatorError::InvalidMultiDelegatePda.into())
    }

    /// Finds the canonical PDA and bump for the multi-delegate account.
    /// Used when creating the multi-delegate to ensure the canonical bump is used.
    pub fn find_pda(user: &Pubkey, token_mint: &Pubkey) -> (Pubkey, u8) {
        find_program_address(
            &[Self::SEED, user.as_ref(), token_mint.as_ref()],
            &crate::ID,
        )
    }
}
