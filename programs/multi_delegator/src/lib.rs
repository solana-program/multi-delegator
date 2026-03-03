//! Multi-Delegator Solana Program.
//!
//! A token delegation program for SPL Token and Token-2022 that allows users to
//! grant scoped spending authority to third parties without transferring ownership.
//!
//! The program supports three delegation models:
//!
//! - **Fixed delegations** -- a one-time allowance with an optional expiry timestamp.
//! - **Recurring delegations** -- a periodic allowance that resets each period, with
//!   configurable period length and overall expiry.
//! - **Subscription plans** -- merchant-defined plans where subscribers grant recurring
//!   pull access; the merchant (or whitelisted pullers) can transfer funds each period.
//!
//! All delegation state is stored in Program Derived Accounts (PDAs). The program is
//! built on the [Pinocchio](https://docs.rs/pinocchio) runtime for minimal compute
//! overhead and uses [Codama](https://github.com/codama-idl/codama) for IDL generation.

use pinocchio::{address::declare_id, AccountView, Address, ProgramResult};

pinocchio::entrypoint!(process_instruction);

pub mod instructions;
pub use instructions::*;

pub mod state;
pub use state::*;

pub mod errors;
pub use errors::*;

pub mod event_engine;
pub mod events;

pub mod constants;
pub use constants::*;

pub mod tests;

declare_id!("EPEUTog1kptYkthDJF6MuB1aM4aDAwHYwoF32Rzv5rqg");

/// Program entrypoint: deserializes the instruction discriminator and dispatches
/// to the appropriate instruction processor.
fn process_instruction(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = MultiDelegatorInstruction::from_bytes(instruction_data)?;

    match instruction {
        MultiDelegatorInstruction::InitMultiDelegate => initialize_multidelegate::process(accounts),
        MultiDelegatorInstruction::CreateFixedDelegation(data) => {
            create_fixed_delegation::process(accounts, &data)
        }
        MultiDelegatorInstruction::CreateRecurringDelegation(data) => {
            create_recurring_delegation::process(accounts, &data)
        }
        MultiDelegatorInstruction::RevokeDelegation => revoke_delegation::process(accounts),
        MultiDelegatorInstruction::TransferFixed(data) => {
            transfer_fixed_delegation::process(accounts, &data)
        }
        MultiDelegatorInstruction::TransferRecurring(data) => {
            transfer_recurring_delegation::process(accounts, &data)
        }
        MultiDelegatorInstruction::CloseMultiDelegate => close_multidelegate::process(accounts),
        MultiDelegatorInstruction::CreatePlan(data) => create_plan::process(accounts, &data),
        MultiDelegatorInstruction::UpdatePlan(data) => update_plan::process(accounts, &data),
        MultiDelegatorInstruction::DeletePlan => delete_plan::process(accounts),
        MultiDelegatorInstruction::TransferSubscription(data) => {
            transfer_subscription::process(accounts, &data)
        }
        MultiDelegatorInstruction::Subscribe(data) => subscribe::process(accounts, &data),
        MultiDelegatorInstruction::CancelSubscription => cancel_subscription::process(accounts),
        MultiDelegatorInstruction::EmitEvent => emit_event::process(program_id, accounts),
    }
}
