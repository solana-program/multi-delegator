pub mod create_fixed_delegation;
pub mod create_recurring_delegation;
pub mod helpers;
pub mod initialize_multidelegate;

pub use helpers::*;

use shank::ShankInstruction;

use crate::create_fixed_delegation::CreateFixedDelegationData;
use crate::create_recurring_delegation::CreateRecurringDelegationData;

#[derive(Debug, ShankInstruction)]
#[repr(u8)]
pub enum MultiDelegatorInstruction {
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

    #[account(
        0,
        signer,
        writable,
        name = "delegator",
        desc = "The user creating the delegation"
    )]
    #[account(
        1,
        writable,
        name = "multi_delegate",
        desc = "The multi_delegate PDA for this token"
    )]
    #[account(
        2,
        writable,
        name = "delegation_account",
        desc = "The fixed delegation PDA being created"
    )]
    #[account(3, name = "delegatee", desc = "The user receiving delegation rights")]
    #[account(4, name = "system_program", desc = "The system program")]
    CreateFixedDelegation(CreateFixedDelegationData) = 1,

    #[account(
        0,
        signer,
        writable,
        name = "delegator",
        desc = "The user creating the delegation"
    )]
    #[account(
        1,
        writable,
        name = "multi_delegate",
        desc = "The multi_delegate PDA for this token"
    )]
    #[account(
        2,
        writable,
        name = "delegation_account",
        desc = "The recurring delegation PDA being created"
    )]
    #[account(3, name = "delegatee", desc = "The user receiving delegation rights")]
    #[account(4, name = "system_program", desc = "The system program")]
    CreateRecurringDelegation(CreateRecurringDelegationData) = 2,
}
