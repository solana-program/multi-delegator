//! Minimal transfer hook for tests and devnet fixtures: `Execute` increments a
//! per-mint counter PDA, proving the hook ran.
#![no_std]

use pinocchio::{
    account::AccountView, default_allocator, error::ProgramError, nostd_panic_handler, program_entrypoint, Address,
    ProgramResult,
};
use transfer_hook_common::{
    create_pda, write_single_meta_list_header, EXECUTE_DISCRIMINATOR, EXTRA_ACCOUNT_METAS_SEED,
    INIT_METAS_DISCRIMINATOR, META_OFFSET, SINGLE_META_VALIDATION_LEN,
};

// Execute accounts: [source, mint, destination, authority, validation, counter]
const COUNTER_ACCOUNT_INDEX: usize = 5;
const MINT_ACCOUNT_INDEX: usize = 1;

const COUNTER_SEED: &[u8] = b"counter";
const COUNTER_LEN: usize = 1;

program_entrypoint!(process_instruction);
default_allocator!();
nostd_panic_handler!();

pub fn process_instruction(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData);
    }
    if instruction_data[..8] == INIT_METAS_DISCRIMINATOR {
        return initialize_extra_account_metas(program_id, accounts);
    }
    if instruction_data[..8] == EXECUTE_DISCRIMINATOR {
        return execute(accounts);
    }
    Err(ProgramError::InvalidInstructionData)
}

fn execute(accounts: &mut [AccountView]) -> ProgramResult {
    let counter = accounts.get_mut(COUNTER_ACCOUNT_INDEX).ok_or(ProgramError::NotEnoughAccountKeys)?;
    let mut data = counter.try_borrow_mut()?;
    let byte = data.first_mut().ok_or(ProgramError::AccountDataTooSmall)?;
    *byte = byte.wrapping_add(1);
    Ok(())
}

// Accounts: [payer, validation PDA, counter PDA, mint, system program]
fn initialize_extra_account_metas(program_id: &Address, accounts: &mut [AccountView]) -> ProgramResult {
    if accounts.len() < 5 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let mut mint_key = [0u8; 32];
    mint_key.copy_from_slice(accounts[3].address().as_ref());

    create_pda(accounts, 1, EXTRA_ACCOUNT_METAS_SEED, &mint_key, program_id, SINGLE_META_VALIDATION_LEN)?;
    write_validation_list(accounts)?;
    create_pda(accounts, 2, COUNTER_SEED, &mint_key, program_id, COUNTER_LEN)?;

    Ok(())
}

// Meta: writable PDA of this program from seeds [Literal("counter"), AccountKey(mint)].
fn write_validation_list(accounts: &mut [AccountView]) -> ProgramResult {
    let mut data = accounts[1].try_borrow_mut()?;
    write_single_meta_list_header(&mut data);
    data[META_OFFSET] = 1; // PDA of this program
    data[META_OFFSET + 1] = 1; // Seed::Literal
    data[META_OFFSET + 2] = COUNTER_SEED.len() as u8;
    data[META_OFFSET + 3..META_OFFSET + 3 + COUNTER_SEED.len()].copy_from_slice(COUNTER_SEED);
    data[META_OFFSET + 10] = 3; // Seed::AccountKey
    data[META_OFFSET + 11] = MINT_ACCOUNT_INDEX as u8;
    data[META_OFFSET + 33] = 0; // is_signer
    data[META_OFFSET + 34] = 1; // is_writable
    Ok(())
}
