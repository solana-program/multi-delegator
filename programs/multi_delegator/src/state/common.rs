use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::{find_program_address, Pubkey};
use shank::ShankType;

use crate::MultiDelegatorError;

pub const DELEGATE_BASE_SEED: &[u8] = b"delegation";

pub fn find_delegation_pda(
    multi_delegate: &Pubkey,
    delegator: &Pubkey,
    delegatee: &Pubkey,
    nonce: u64,
) -> (Pubkey, u8) {
    find_program_address(
        &[
            DELEGATE_BASE_SEED,
            multi_delegate.as_ref(),
            delegator.as_ref(),
            delegatee.as_ref(),
            &nonce.to_le_bytes(),
        ],
        &crate::ID,
    )
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug, ShankType)]
pub enum DelegationKind {
    Fixed = 0,
    Recurring = 1,
}

impl TryFrom<u8> for DelegationKind {
    type Error = ProgramError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Fixed),
            1 => Ok(Self::Recurring),
            _ => Err(MultiDelegatorError::InvalidDelegationKind.into()),
        }
    }
}

impl From<DelegationKind> for u8 {
    fn from(val: DelegationKind) -> Self {
        match val {
            DelegationKind::Fixed => 0,
            DelegationKind::Recurring => 1,
        }
    }
}
