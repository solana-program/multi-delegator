pub const TOKEN_2022_PROGRAM_ID: pinocchio::Address = pinocchio_token_2022::ID;

pub const TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET: usize = 165;
pub const TOKEN_2022_MINT_DISCRIMINATOR: u8 = 0x01;
pub const TOKEN_2022_TOKEN_ACCOUNT_DISCRIMINATOR: u8 = 0x02;

/// SPL Token account layout: the owner field starts at byte 32 and is 32 bytes long.
pub const TOKEN_ACCOUNT_OWNER_OFFSET: usize = 32;
pub const TOKEN_ACCOUNT_OWNER_END: usize = TOKEN_ACCOUNT_OWNER_OFFSET + 32;
