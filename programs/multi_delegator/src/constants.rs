// --- Token program ---
pub const TOKEN_2022_PROGRAM_ID: pinocchio::Address = pinocchio_token_2022::ID;

// --- Shared business logic limits ---
pub const TIME_DRIFT_ALLOWED_SECS: i64 = 120; // seconds

// --- Token account discriminators ---
pub const TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET: usize = 165;
pub const TOKEN_2022_MINT_DISCRIMINATOR: u8 = 0x01;
pub const TOKEN_2022_TOKEN_ACCOUNT_DISCRIMINATOR: u8 = 0x02;

// --- Token account layout offsets ---
pub const TOKEN_ACCOUNT_MINT_OFFSET: usize = 0;
pub const TOKEN_ACCOUNT_MINT_END: usize = TOKEN_ACCOUNT_MINT_OFFSET + 32;
pub const TOKEN_ACCOUNT_OWNER_OFFSET: usize = 32;
pub const TOKEN_ACCOUNT_OWNER_END: usize = TOKEN_ACCOUNT_OWNER_OFFSET + 32;
