use codama::CodamaType;
use pinocchio::pubkey::Pubkey;

pub const VERSION_OFFSET: usize = 0;
pub const KIND_OFFSET: usize = 1;
pub const BUMP_OFFSET: usize = 2;
pub const DELEGATOR_OFFSET: usize = 3;
pub const DELEGATEE_OFFSET: usize = 35;
pub const PAYER_OFFSET: usize = 67;
pub const CURRENT_VERSION: u8 = 1;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, CodamaType)]
pub struct Header {
    pub version: u8,
    pub kind: u8,
    pub bump: u8,
    pub delegator: Pubkey,
    pub delegatee: Pubkey,
    pub payer: Pubkey,
}

impl Header {
    pub const LEN: usize = core::mem::size_of::<Header>();
}
