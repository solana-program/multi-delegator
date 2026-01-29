use core::mem::{size_of, transmute};
use pinocchio::{program_error::ProgramError, pubkey::find_program_address, pubkey::Pubkey};
use shank::ShankAccount;

use crate::MultiDelegatorError;

#[repr(C)]
#[derive(ShankAccount)]
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

    pub fn find_pda(user: &Pubkey, token_mint: &Pubkey) -> (Pubkey, u8) {
        find_program_address(
            &[Self::SEED, user.as_ref(), token_mint.as_ref()],
            &crate::ID,
        )
    }
}
