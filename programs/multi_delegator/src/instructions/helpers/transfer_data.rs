use pinocchio::{program_error::ProgramError, pubkey::Pubkey};
use shank::ShankType;

use crate::MultiDelegatorError;

#[repr(C, packed)]
#[derive(ShankType, Debug)]
pub struct TransferData {
    pub amount: u64,
    pub delegator: Pubkey,
    pub mint: Pubkey,
}

impl TransferData {
    pub const LEN: usize = size_of::<TransferData>();
    pub const DELEGATOR_OFFSET: usize = 8;
    pub const MINT_OFFSET: usize = Self::DELEGATOR_OFFSET + 32;

    pub fn load(data: &[u8]) -> Result<Self, ProgramError> {
        if data.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidInstructionData.into());
        }
        let amount = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        let delegator: Pubkey = data[Self::DELEGATOR_OFFSET..Self::MINT_OFFSET]
            .try_into()
            .map_err(|_| MultiDelegatorError::TransferInvalidCallData)?;

        let mint = data[Self::MINT_OFFSET..Self::MINT_OFFSET + 32]
            .try_into()
            .map_err(|_| MultiDelegatorError::TransferInvalidCallData)?;

        Ok(Self {
            amount,
            delegator,
            mint,
        })
    }
}
