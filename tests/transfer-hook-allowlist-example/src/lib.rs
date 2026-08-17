//! Token-2022 transfer-hook example enforcing a destination-owner allowlist on
//! delegated transfers. `Execute` classifies the transfer authority:
//! - the source token account's owner passes unconditionally,
//! - the subscriptions program's SubscriptionAuthority PDA passes only when an
//!   allow-entry PDA exists for the destination token account's owner,
//! - every other delegate is rejected.
//! `InitializeExtraAccountMetaList` writes the validation PDA whose single meta
//! derives the allow-entry from the destination owner (AccountData seed), so
//! Token-2022 resolves and forwards it on every transfer.
//! `InitializeAllowEntry` creates the allow-entry PDA for an approved owner.
#![no_std]

use pinocchio::{
    account::AccountView,
    address::address,
    cpi::{Seed, Signer},
    default_allocator,
    error::ProgramError,
    nostd_panic_handler, program_entrypoint,
    sysvars::{rent::Rent, Sysvar},
    Address, ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

// sha256("spl-transfer-hook-interface:execute")[..8]
const EXECUTE_DISCRIMINATOR: [u8; 8] = [0x69, 0x25, 0x65, 0xc5, 0x4b, 0xfb, 0x66, 0x1a];
// sha256("spl-transfer-hook-interface:initialize-extra-account-metas")[..8]
const INIT_METAS_DISCRIMINATOR: [u8; 8] = [43, 34, 13, 49, 167, 88, 235, 235];
const INIT_ALLOW_DISCRIMINATOR: [u8; 8] = *b"initallo";

const SUBSCRIPTIONS_PROGRAM_ID: Address = address!("De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44");
const SUBSCRIPTION_AUTHORITY_SEED: &[u8] = b"SubscriptionAuthority";

const EXTRA_ACCOUNT_METAS_SEED: &[u8] = b"extra-account-metas";
const ALLOW_SEED: &[u8] = b"allow";

const TOKEN_ACCOUNT_OWNER_OFFSET: usize = 32;
const TOKEN_ACCOUNT_OWNER_END: usize = 64;

// Execute accounts: [source, mint, destination, authority, validation, allow_entry]
const DESTINATION_ACCOUNT_INDEX: usize = 2;
const ALLOW_ENTRY_ACCOUNT_INDEX: usize = 5;

// ExtraAccountMetaList with one seed-derived allow-entry meta: 8-byte execute
// discriminator, u32 value length (4 + 35), u32 entry count (1), one 35-byte meta.
const VALIDATION_LEN: usize = 51;
const ALLOW_ENTRY_LEN: usize = 1;

const ERROR_DELEGATE_NOT_ALLOWED: u32 = 1;
const ERROR_DESTINATION_NOT_ALLOWED: u32 = 2;

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
    if instruction_data[..8] == INIT_ALLOW_DISCRIMINATOR {
        return initialize_allow_entry(program_id, accounts, instruction_data);
    }
    if instruction_data[..8] == EXECUTE_DISCRIMINATOR {
        return execute(program_id, accounts);
    }
    Err(ProgramError::InvalidInstructionData)
}

fn token_account_owner(account: &AccountView) -> Result<[u8; 32], ProgramError> {
    let data = account.try_borrow()?;
    let bytes =
        data.get(TOKEN_ACCOUNT_OWNER_OFFSET..TOKEN_ACCOUNT_OWNER_END).ok_or(ProgramError::InvalidAccountData)?;
    let mut owner = [0u8; 32];
    owner.copy_from_slice(bytes);
    Ok(owner)
}

fn execute(program_id: &Address, accounts: &[AccountView]) -> ProgramResult {
    let [source, mint, _, authority, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let source_owner = token_account_owner(source)?;
    if *authority.address() == Address::new_from_array(source_owner) {
        return Ok(());
    }

    let (subscription_authority, _) = Address::find_program_address(
        &[SUBSCRIPTION_AUTHORITY_SEED, &source_owner, mint.address().as_ref()],
        &SUBSCRIPTIONS_PROGRAM_ID,
    );
    if *authority.address() != subscription_authority {
        return Err(ProgramError::Custom(ERROR_DELEGATE_NOT_ALLOWED));
    }

    let destination = &accounts[DESTINATION_ACCOUNT_INDEX];
    let allow_entry = accounts.get(ALLOW_ENTRY_ACCOUNT_INDEX).ok_or(ProgramError::NotEnoughAccountKeys)?;
    let destination_owner = token_account_owner(destination)?;
    let (expected_entry, _) = Address::find_program_address(&[ALLOW_SEED, &destination_owner], program_id);
    if *allow_entry.address() != expected_entry || !allow_entry.owned_by(program_id) {
        return Err(ProgramError::Custom(ERROR_DESTINATION_NOT_ALLOWED));
    }
    Ok(())
}

// Accounts: [payer, validation PDA, mint, system program]
fn initialize_extra_account_metas(program_id: &Address, accounts: &mut [AccountView]) -> ProgramResult {
    if accounts.len() < 4 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let mut mint_key = [0u8; 32];
    mint_key.copy_from_slice(accounts[2].address().as_ref());

    create_pda(accounts, 1, EXTRA_ACCOUNT_METAS_SEED, &mint_key, program_id, VALIDATION_LEN)?;
    write_validation_list(accounts)
}

// Accounts: [payer, allow-entry PDA, system program]; data[8..40] = approved owner.
fn initialize_allow_entry(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    if accounts.len() < 3 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    if instruction_data.len() < 40 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let mut owner = [0u8; 32];
    owner.copy_from_slice(&instruction_data[8..40]);

    create_pda(accounts, 1, ALLOW_SEED, &owner, program_id, ALLOW_ENTRY_LEN)
}

fn create_pda(
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

// Meta: PDA of the hook program from seeds [Literal("allow"),
// AccountData(destination.owner)], readonly non-signer.
fn write_validation_list(accounts: &mut [AccountView]) -> ProgramResult {
    let mut data = accounts[1].try_borrow_mut()?;
    data[..8].copy_from_slice(&EXECUTE_DISCRIMINATOR);
    data[8..12].copy_from_slice(&((4 + 35) as u32).to_le_bytes());
    data[12..16].copy_from_slice(&1u32.to_le_bytes());
    data[16] = 1; // ExtraAccountMeta discriminator: PDA of the hook program
    data[17] = 1; // seed 0: Literal
    data[18] = ALLOW_SEED.len() as u8;
    data[19..19 + ALLOW_SEED.len()].copy_from_slice(ALLOW_SEED);
    data[24] = 4; // seed 1: AccountData
    data[25] = DESTINATION_ACCOUNT_INDEX as u8;
    data[26] = TOKEN_ACCOUNT_OWNER_OFFSET as u8;
    data[27] = 32; // owner length
    data[49] = 0; // is_signer
    data[50] = 0; // is_writable
    Ok(())
}
