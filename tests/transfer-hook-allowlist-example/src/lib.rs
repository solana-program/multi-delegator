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
    account::AccountView, address::address, default_allocator, error::ProgramError, nostd_panic_handler,
    program_entrypoint, Address, ProgramResult,
};
use transfer_hook_common::{
    create_pda, write_single_meta_list_header, EXECUTE_DISCRIMINATOR, EXTRA_ACCOUNT_METAS_SEED,
    INIT_METAS_DISCRIMINATOR, META_OFFSET, SINGLE_META_VALIDATION_LEN,
};

const INIT_ALLOW_DISCRIMINATOR: [u8; 8] = *b"initallo";

const SUBSCRIPTIONS_PROGRAM_ID: Address = address!("De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44");
const SUBSCRIPTION_AUTHORITY_SEED: &[u8] = b"SubscriptionAuthority";

const ALLOW_SEED: &[u8] = b"allow";

const TOKEN_ACCOUNT_OWNER_OFFSET: usize = 32;
const TOKEN_ACCOUNT_OWNER_END: usize = 64;

// Execute accounts: [source, mint, destination, authority, validation, allow_entry]
const DESTINATION_ACCOUNT_INDEX: usize = 2;
const ALLOW_ENTRY_ACCOUNT_INDEX: usize = 5;

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

    create_pda(accounts, 1, EXTRA_ACCOUNT_METAS_SEED, &mint_key, program_id, SINGLE_META_VALIDATION_LEN)?;
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

// Meta: PDA of the hook program from seeds [Literal("allow"),
// AccountData(destination.owner)], readonly non-signer.
fn write_validation_list(accounts: &mut [AccountView]) -> ProgramResult {
    let mut data = accounts[1].try_borrow_mut()?;
    write_single_meta_list_header(&mut data);
    data[META_OFFSET] = 1; // ExtraAccountMeta discriminator: PDA of the hook program
    data[META_OFFSET + 1] = 1; // seed 0: Literal
    data[META_OFFSET + 2] = ALLOW_SEED.len() as u8;
    data[META_OFFSET + 3..META_OFFSET + 3 + ALLOW_SEED.len()].copy_from_slice(ALLOW_SEED);
    data[META_OFFSET + 8] = 4; // seed 1: AccountData
    data[META_OFFSET + 9] = DESTINATION_ACCOUNT_INDEX as u8;
    data[META_OFFSET + 10] = TOKEN_ACCOUNT_OWNER_OFFSET as u8;
    data[META_OFFSET + 11] = 32; // owner length
    data[META_OFFSET + 33] = 0; // is_signer
    data[META_OFFSET + 34] = 0; // is_writable
    Ok(())
}
