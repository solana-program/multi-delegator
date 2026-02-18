use pinocchio::{address::declare_id, AccountView, Address, ProgramResult};

pinocchio::entrypoint!(process_instruction);

pub mod instructions;
pub use instructions::*;

pub mod state;
pub use state::*;

pub mod errors;
pub use errors::*;

pub mod constants;
pub use constants::*;

pub mod tests;

declare_id!("3PuMsYqaLY4Sy1DR8np3aAiHravZXCeyMYDUECLqfswY");

fn process_instruction(
    _program_id: &Address,
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
    }
}
