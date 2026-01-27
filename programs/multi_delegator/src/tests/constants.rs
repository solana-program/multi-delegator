use solana_pubkey::Pubkey;

pub static SYSTEM_PROGRAM_ID: Pubkey = Pubkey::new_from_array(pinocchio_system::ID);
pub static TOKEN_PROGRAM_ID: Pubkey = Pubkey::new_from_array(pinocchio_token::ID);
pub static TOKEN_2022_PROGRAM_ID: Pubkey = Pubkey::new_from_array(pinocchio_token_2022::ID);
pub static ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    Pubkey::new_from_array(pinocchio_associated_token_account::ID);

pub static PROGRAM_ID: Pubkey = Pubkey::new_from_array(crate::ID);
pub const MINT_DECIMALS: u8 = 6;
