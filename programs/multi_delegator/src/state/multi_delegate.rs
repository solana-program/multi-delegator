use codama::CodamaAccount;
use core::mem::{size_of, transmute};
use pinocchio::{error::ProgramError, Address};

use crate::{state::common::AccountDiscriminator, MultiDelegatorError};

#[repr(C)]
#[derive(CodamaAccount)]
pub struct MultiDelegate {
    pub discriminator: u8,
    pub user: Address,
    pub token_mint: Address,
    pub bump: u8,
}

impl MultiDelegate {
    pub const LEN: usize = size_of::<MultiDelegate>();
    pub const SEED: &'static [u8] = b"MultiDelegate";

    /// Initializes a freshly created account by setting all fields including
    /// the discriminator. Use this instead of `load_mut` when the account
    /// has just been created.
    #[inline(always)]
    pub fn init<'a>(
        bytes: &'a mut [u8],
        user: &Address,
        token_mint: &Address,
        bump: u8,
    ) -> Result<&'a Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        let account = unsafe { &mut *transmute::<*mut u8, *mut Self>(bytes.as_mut_ptr()) };
        account.discriminator = AccountDiscriminator::MultiDelegate as u8;
        account.user = *user;
        account.token_mint = *token_mint;
        account.bump = bump;
        Ok(account)
    }

    #[inline(always)]
    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        if bytes[0] != AccountDiscriminator::MultiDelegate as u8 {
            return Err(MultiDelegatorError::InvalidAccountDiscriminator.into());
        }
        Ok(unsafe { &mut *transmute::<*mut u8, *mut Self>(bytes.as_mut_ptr()) })
    }

    #[inline(always)]
    pub fn load(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        if bytes[0] != AccountDiscriminator::MultiDelegate as u8 {
            return Err(MultiDelegatorError::InvalidAccountDiscriminator.into());
        }
        Ok(unsafe { &*transmute::<*const u8, *const Self>(bytes.as_ptr()) })
    }

    pub fn check_owner(&self, expected_user: &Address) -> Result<(), ProgramError> {
        if self.user != *expected_user {
            return Err(MultiDelegatorError::Unauthorized.into());
        }
        Ok(())
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
