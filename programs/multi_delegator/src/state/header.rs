use pinocchio::pubkey::Pubkey;
use shank::ShankType;

pub const VERSION_OFFSET: usize = 0;
pub const KIND_OFFSET: usize = 1;
pub const CURRENT_VERSION: u8 = 1;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, ShankType)]
pub struct Header {
    pub version: u8,
    pub kind: u8,
    pub bump: u8,
    pub delegator: Pubkey,
    pub delegatee: Pubkey,
}

impl Header {
    pub const LEN: usize = core::mem::size_of::<Header>();
}
