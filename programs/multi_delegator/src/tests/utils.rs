use std::time::{SystemTime, UNIX_EPOCH};

use litesvm::{types::TransactionResult, LiteSVM};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_associated_token_account::{
    get_associated_token_address,
    solana_program::{
        clock::Clock,
        native_token::LAMPORTS_PER_SOL,
        program_pack::{IsInitialized, Pack},
    },
};
use spl_token_2022::state::{Account as TokenAccount, AccountState, Mint as Mint2022};

use solana_instruction::AccountMeta;

use crate::{
    instructions::{
        create_fixed_delegation, create_recurring_delegation, initialize_multidelegate,
        revoke_delegation, transfer_fixed_delegation, transfer_recurring_delegation,
    },
    tests::{
        constants::{PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID},
        pda::{get_delegation_pda, get_multidelegate_pda},
    },
};

/// Converts number of minutes into seconds
pub fn minutes(mins: u64) -> u64 {
    mins * 60
}

/// Converts number of hours into seconds
pub fn hours(hours: u64) -> u64 {
    hours * minutes(60)
}

/// Converts number of days into seconds
pub fn days(days: u64) -> u64 {
    days * hours(24)
}

pub fn current_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn move_clock_forward(litesvm: &mut LiteSVM, seconds: u64) {
    let mut initial_clock = litesvm.get_sysvar::<Clock>();
    initial_clock.unix_timestamp += seconds as i64;
    litesvm.set_sysvar::<Clock>(&initial_clock);
}

pub fn get_ata_balance(litesvm: &LiteSVM, ata: &Pubkey) -> u64 {
    let account = fetch_account::<spl_token_2022::state::Account>(litesvm, ata);
    account.amount
}

pub fn setup() -> (LiteSVM, Keypair) {
    let mut litesvm = LiteSVM::new();

    litesvm
        .add_program_from_file(PROGRAM_ID.to_bytes(), "target/deploy/multi_delegator.so")
        .unwrap();

    let mut initial_clock = litesvm.get_sysvar::<Clock>();
    initial_clock.unix_timestamp = current_ts();
    litesvm.set_sysvar::<Clock>(&initial_clock);

    let default_payer = Keypair::new();
    litesvm
        .airdrop(&default_payer.pubkey(), LAMPORTS_PER_SOL * 100)
        .unwrap();

    (litesvm, default_payer)
}

fn pack_data<T: Pack>(state: T) -> Vec<u8> {
    let mut data = vec![0; T::LEN];
    T::pack(state, &mut data).unwrap();
    data
}

pub fn fetch_account<T: Pack + IsInitialized>(litesvm: &LiteSVM, pubkey: &Pubkey) -> T {
    let account = litesvm.get_account(pubkey).unwrap();
    T::unpack(account.data.as_ref()).unwrap()
}

#[allow(clippy::result_large_err)]
pub fn build_and_send_transaction(
    litesvm: &mut LiteSVM,
    signers: &[&Keypair],
    payer: &Pubkey,
    ixs: &[Instruction],
) -> TransactionResult {
    let tx = Transaction::new(
        signers,
        Message::new(ixs, Some(payer)),
        litesvm.latest_blockhash(),
    );
    let result = litesvm.send_transaction(tx);
    litesvm.expire_blockhash();
    result
}

pub fn init_wallet(litesvm: &mut LiteSVM, lamports: u64) -> Keypair {
    let wallet = Keypair::new();
    litesvm.airdrop(&wallet.pubkey(), lamports).unwrap();
    wallet
}

pub fn init_mint(
    litesvm: &mut LiteSVM,
    token_program: Pubkey,
    decimals: u8,
    supply: u64,
    authority: Option<Pubkey>,
) -> Pubkey {
    let mint = Pubkey::new_unique();

    let mint_state = Mint2022 {
        mint_authority: authority.into(),
        supply,
        decimals,
        is_initialized: true,
        freeze_authority: None.into(),
    };

    let mint_data = pack_data(mint_state);
    let lamports = litesvm.minimum_balance_for_rent_exemption(Mint2022::LEN);

    litesvm
        .set_account(
            mint,
            Account {
                lamports,
                data: mint_data,
                owner: token_program,
                executable: false,
                rent_epoch: 0,
            },
        )
        .unwrap();

    mint
}

pub fn init_ata(litesvm: &mut LiteSVM, mint: Pubkey, owner: Pubkey, amount: u64) -> Pubkey {
    let token_program = litesvm.get_account(&mint).unwrap().owner;
    let ata = get_associated_token_address(&owner, &mint);

    let ata_state = TokenAccount {
        mint,
        owner,
        amount,
        delegate: None.into(),
        state: AccountState::Initialized,
        is_native: None.into(),
        delegated_amount: 0,
        close_authority: None.into(),
    };

    let ata_data = pack_data(ata_state);
    let lamports = litesvm.minimum_balance_for_rent_exemption(TokenAccount::LEN);

    litesvm
        .set_account(
            ata,
            Account {
                lamports,
                data: ata_data,
                owner: token_program,
                executable: false,
                rent_epoch: 0,
            },
        )
        .unwrap();

    ata
}

pub fn initialize_multidelegate_action(
    litesvm: &mut LiteSVM,
    payer: &Keypair,
    mint: Pubkey,
) -> (TransactionResult, Pubkey, u8) {
    let token_program = litesvm.get_account(&mint).unwrap().owner;
    let user_ata = get_associated_token_address(&payer.pubkey(), &mint);
    let (multi_delegate_pda, bump) = get_multidelegate_pda(&payer.pubkey(), &mint);

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(multi_delegate_pda, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(user_ata, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(token_program, false),
        ],
        data: vec![*initialize_multidelegate::DISCRIMINATOR],
    };

    (
        build_and_send_transaction(litesvm, &[payer], &payer.pubkey(), &[ix]),
        multi_delegate_pda,
        bump,
    )
}

pub fn create_fixed_delegation_action(
    litesvm: &mut LiteSVM,
    delegator: &Keypair,
    mint: Pubkey,
    delegatee: Pubkey,
    nonce: u64,
    amount: u64,
    expiry_ts: i64,
) -> (TransactionResult, Pubkey) {
    let (multi_delegate_pda, _bump) = get_multidelegate_pda(&delegator.pubkey(), &mint);
    let (delegation_pda, _bump) =
        get_delegation_pda(&multi_delegate_pda, &delegator.pubkey(), &delegatee, nonce);

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(delegator.pubkey(), true),
            AccountMeta::new(multi_delegate_pda, false),
            AccountMeta::new(delegation_pda, false),
            AccountMeta::new_readonly(delegatee, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: [
            vec![*create_fixed_delegation::DISCRIMINATOR],
            nonce.to_le_bytes().to_vec(),
            amount.to_le_bytes().to_vec(),
            expiry_ts.to_le_bytes().to_vec(),
        ]
        .concat(),
    };

    (
        build_and_send_transaction(litesvm, &[delegator], &delegator.pubkey(), &[ix]),
        delegation_pda,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn create_recurring_delegation_action(
    litesvm: &mut LiteSVM,
    delegator: &Keypair,
    mint: Pubkey,
    delegatee: Pubkey,
    nonce: u64,
    amount_per_period: u64,
    period_length_s: u64,
    start_ts: i64,
    expiry_ts: i64,
) -> (TransactionResult, Pubkey) {
    let (multi_delegate_pda, _bump) = get_multidelegate_pda(&delegator.pubkey(), &mint);
    let (delegation_pda, _bump) =
        get_delegation_pda(&multi_delegate_pda, &delegator.pubkey(), &delegatee, nonce);

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(delegator.pubkey(), true),
            AccountMeta::new(multi_delegate_pda, false),
            AccountMeta::new(delegation_pda, false),
            AccountMeta::new_readonly(delegatee, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: [
            vec![*create_recurring_delegation::DISCRIMINATOR],
            nonce.to_le_bytes().to_vec(),
            amount_per_period.to_le_bytes().to_vec(),
            period_length_s.to_le_bytes().to_vec(),
            start_ts.to_le_bytes().to_vec(),
            expiry_ts.to_le_bytes().to_vec(),
        ]
        .concat(),
    };

    (
        build_and_send_transaction(litesvm, &[delegator], &delegator.pubkey(), &[ix]),
        delegation_pda,
    )
}

#[allow(clippy::too_many_arguments, clippy::result_large_err)]
pub fn create_fixed_delegation_action_with_pda(
    litesvm: &mut LiteSVM,
    payer: &Keypair,
    mint: Pubkey,
    delegatee: Pubkey,
    delegation_pda: Pubkey,
    nonce: u64,
    amount: u64,
    expiry_s: u64,
) -> TransactionResult {
    let (multi_delegate_pda, _bump) = get_multidelegate_pda(&payer.pubkey(), &mint);

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(multi_delegate_pda, false),
            AccountMeta::new(delegation_pda, false),
            AccountMeta::new_readonly(delegatee, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: [
            vec![*create_fixed_delegation::DISCRIMINATOR],
            nonce.to_le_bytes().to_vec(),
            amount.to_le_bytes().to_vec(),
            expiry_s.to_le_bytes().to_vec(),
        ]
        .concat(),
    };

    build_and_send_transaction(litesvm, &[payer], &payer.pubkey(), &[ix])
}

#[allow(clippy::result_large_err)]
pub fn revoke_delegation_action(
    litesvm: &mut LiteSVM,
    payer: &Keypair,
    mint: Pubkey,
    delegatee: Pubkey,
    nonce: u64,
) -> TransactionResult {
    let (multi_delegate_pda, _bump) = get_multidelegate_pda(&payer.pubkey(), &mint);
    let (delegation_pda, _bump) =
        get_delegation_pda(&multi_delegate_pda, &payer.pubkey(), &delegatee, nonce);

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(delegation_pda, false),
        ],
        data: vec![*revoke_delegation::DISCRIMINATOR],
    };

    build_and_send_transaction(litesvm, &[payer], &payer.pubkey(), &[ix])
}

#[allow(clippy::result_large_err)]
pub fn transfer_fixed_action_with_signer(
    litesvm: &mut LiteSVM,
    delegator_pubkey: Pubkey,
    mint: Pubkey,
    delegation_pda: Pubkey,
    amount: u64,
    receiver_ata: Pubkey,
    signer: &Keypair,
) -> TransactionResult {
    let (multi_delegate_pda, _bump) = get_multidelegate_pda(&delegator_pubkey, &mint);
    let delegator_ata = get_associated_token_address(&delegator_pubkey, &mint);

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(delegation_pda, false),
            AccountMeta::new(multi_delegate_pda, false),
            AccountMeta::new(delegator_ata, false),
            AccountMeta::new(receiver_ata, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(signer.pubkey(), true),
        ],
        data: [
            vec![*transfer_fixed_delegation::DISCRIMINATOR],
            amount.to_le_bytes().to_vec(),
            delegator_pubkey.to_bytes().to_vec(),
            mint.to_bytes().to_vec(),
        ]
        .concat(),
    };

    build_and_send_transaction(litesvm, &[signer], &signer.pubkey(), &[ix])
}

#[allow(clippy::result_large_err)]
pub fn transfer_fixed_action(
    litesvm: &mut LiteSVM,
    delegator: &Keypair,
    delegatee: &Keypair,
    mint: Pubkey,
    delegation_pda: Pubkey,
    amount: u64,
) -> TransactionResult {
    let delegatee_ata = get_associated_token_address(&delegatee.pubkey(), &mint);

    transfer_fixed_action_with_signer(
        litesvm,
        delegator.pubkey(),
        mint,
        delegation_pda,
        amount,
        delegatee_ata,
        delegatee,
    )
}

#[allow(clippy::result_large_err)]
pub fn transfer_recurring_action_with_signer(
    litesvm: &mut LiteSVM,
    delegator_pubkey: Pubkey,
    mint: Pubkey,
    delegation_pda: Pubkey,
    amount: u64,
    receiver_ata: Pubkey,
    signer: &Keypair,
) -> TransactionResult {
    let (multi_delegate_pda, _bump) = get_multidelegate_pda(&delegator_pubkey, &mint);
    let delegator_ata = get_associated_token_address(&delegator_pubkey, &mint);

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(delegation_pda, false),
            AccountMeta::new(multi_delegate_pda, false),
            AccountMeta::new(delegator_ata, false),
            AccountMeta::new(receiver_ata, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(signer.pubkey(), true),
        ],
        data: [
            vec![*transfer_recurring_delegation::DISCRIMINATOR],
            amount.to_le_bytes().to_vec(),
            delegator_pubkey.to_bytes().to_vec(),
            mint.to_bytes().to_vec(),
        ]
        .concat(),
    };

    build_and_send_transaction(litesvm, &[signer], &signer.pubkey(), &[ix])
}

#[allow(clippy::result_large_err)]
pub fn transfer_recurring_action(
    litesvm: &mut LiteSVM,
    delegator: &Keypair,
    delegatee: &Keypair,
    mint: Pubkey,
    delegation_pda: Pubkey,
    amount: u64,
) -> TransactionResult {
    let delegatee_ata = get_associated_token_address(&delegatee.pubkey(), &mint);

    transfer_recurring_action_with_signer(
        litesvm,
        delegator.pubkey(),
        mint,
        delegation_pda,
        amount,
        delegatee_ata,
        delegatee,
    )
}
