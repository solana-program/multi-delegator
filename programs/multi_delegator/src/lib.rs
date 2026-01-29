use pinocchio::{account_info::AccountInfo, entrypoint, pubkey::Pubkey, ProgramResult};

entrypoint!(process_instruction);

pub mod instructions;
pub use instructions::*;

pub mod state;
use pinocchio_pubkey::declare_id;
pub use state::*;

pub mod errors;
pub use errors::*;

pub mod constants;
pub use constants::*;

pub mod tests;

declare_id!("3PuMsYqaLY4Sy1DR8np3aAiHravZXCeyMYDUECLqfswY");

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((0, data)) => initialize_multidelegate::process((data, accounts)),
        Some((1, data)) => create_fixed_delegation::process((data, accounts)),
        Some((2, data)) => create_recurring_delegation::process((data, accounts)),
        _ => Err(MultiDelegatorError::InvalidInstruction.into()),
    }
}
