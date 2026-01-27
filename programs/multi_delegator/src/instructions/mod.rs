pub mod add_delegate;
pub mod helpers;
pub mod initialize_multidelegate;

pub use helpers::*;

use shank::ShankInstruction;

use crate::add_delegate::AddDelegateInstructionData;
#[derive(Debug, ShankInstruction)]
#[repr(u8)]
pub enum VaultInstruction {
    /// Initialize a token vault, starts inactivate. Add tokens in subsequent instructions, then activate.
    #[account(
        0,
        signer,
        writable,
        name = "owner",
        desc = "The owner of the multi-delegate program"
    )]
    #[account(
        1,
        writable,
        name = "multi_delegate",
        desc = "The multi_delegate PDA that will be the delegate instance for this token"
    )]
    #[account(
        2,
        name = "token_mint",
        desc = "The token mint that we are creating a multi delegate account for"
    )]
    #[account(
        3,
        writable,
        name = "user_ata",
        desc = "The ata that we are setting up delegation for"
    )]
    #[account(4, name = "system_program", desc = "The system program")]
    #[account(5, name = "token_program", desc = "Token program")]
    InitMultiDelegate = 0,

    /// Initialize a token vault, starts inactivate. Add tokens in subsequent instructions, then activate.
    #[account(
        0,
        signer,
        writable,
        name = "user",
        desc = "The user user who is initializing the multidelegate instance for a particular token"
    )]
    #[account(
        1,
        writable,
        name = "multi_delegate",
        desc = "The multi_delegate PDA that will be the delegate instance for this token"
    )]
    #[account(
        2,
        name = "delegate_account",
        desc = "The account which will be keeping track of how much the delegate is able to pull"
    )]
    #[account(
        3,
        name = "delegate",
        desc = "The user is being designated as a delegate. They will be able to spend the users tokens"
    )]
    #[account(4, name = "system_program", desc = "The system program")]
    CreateSimpleDelegation(AddDelegateInstructionData) = 1,
}
