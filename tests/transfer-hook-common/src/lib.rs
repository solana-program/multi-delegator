//! Shared constants and helpers for the example transfer-hook programs.
#![no_std]

use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

// sha256("spl-transfer-hook-interface:execute")[..8]
pub const EXECUTE_DISCRIMINATOR: [u8; 8] = [0x69, 0x25, 0x65, 0xc5, 0x4b, 0xfb, 0x66, 0x1a];
// sha256("spl-transfer-hook-interface:initialize-extra-account-metas")[..8]
pub const INIT_METAS_DISCRIMINATOR: [u8; 8] = [43, 34, 13, 49, 167, 88, 235, 235];

pub const EXTRA_ACCOUNT_METAS_SEED: &[u8] = b"extra-account-metas";

// ExtraAccountMetaList with one 35-byte meta: 8-byte execute discriminator,
// u32 value length (4 + 35), u32 entry count (1), the meta.
pub const SINGLE_META_VALIDATION_LEN: usize = 51;
pub const META_OFFSET: usize = 16;

/// Writes the `ExtraAccountMetaList` header for a single-meta list; the caller
/// fills the 35-byte meta at [`META_OFFSET`].
pub fn write_single_meta_list_header(data: &mut [u8]) {
    data[..8].copy_from_slice(&EXECUTE_DISCRIMINATOR);
    data[8..12].copy_from_slice(&((4 + 35) as u32).to_le_bytes());
    data[12..16].copy_from_slice(&1u32.to_le_bytes());
}

/// Creates a PDA of `program_id` at `accounts[account_index]` from seeds
/// `[prefix, key]`, funded by `accounts[0]`.
pub fn create_pda(
    accounts: &mut [AccountView],
    account_index: usize,
    prefix: &[u8],
    key: &[u8; 32],
    program_id: &Address,
    space: usize,
) -> ProgramResult {
    let (expected_pda, bump) = Address::find_program_address(&[prefix, key.as_ref()], program_id);
    if expected_pda != *accounts[account_index].address() {
        return Err(ProgramError::InvalidInstructionData);
    }

    let lamports = Rent::get()?.try_minimum_balance(space)?;
    let bump_binding = [bump];
    let seeds = [Seed::from(prefix), Seed::from(key.as_ref()), Seed::from(&bump_binding)];
    let signer = [Signer::from(&seeds)];

    CreateAccount { from: &accounts[0], to: &accounts[account_index], lamports, space: space as u64, owner: program_id }
        .invoke_signed(&signer)
}
