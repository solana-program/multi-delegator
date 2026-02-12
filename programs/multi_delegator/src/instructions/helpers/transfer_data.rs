use codama::CodamaType;
use core::mem::{size_of, transmute};
use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

use crate::MultiDelegatorError;

#[repr(C, packed)]
#[derive(CodamaType, Debug, Clone)]
pub struct TransferData {
    pub amount: u64,
    pub delegator: Pubkey,
    pub mint: Pubkey,
}

impl TransferData {
    pub const LEN: usize = size_of::<TransferData>();

    pub fn load(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidInstructionData.into());
        }
        Ok(unsafe { &*transmute::<*const u8, *const Self>(data.as_ptr()) })
    }
}
