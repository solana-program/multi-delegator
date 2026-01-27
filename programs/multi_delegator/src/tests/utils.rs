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
        native_token::LAMPORTS_PER_SOL,
        program_pack::{IsInitialized, Pack},
    },
};
use spl_token_2022::state::{Account as TokenAccount, AccountState, Mint as Mint2022};

use solana_instruction::AccountMeta;

use crate::{
    instructions::{add_delegate, initialize_multidelegate},
    tests::{
        constants::{PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID},
        pda::{get_delegate_pda, get_multidelegate_pda},
    },
    TermsKind,
};

pub fn setup() -> (LiteSVM, Keypair) {
    let mut litesvm = LiteSVM::new();

    litesvm
        .add_program_from_file(PROGRAM_ID.to_bytes(), "target/deploy/multi_delegator.so")
        .unwrap();

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
    litesvm.send_transaction(tx)
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
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data: vec![*initialize_multidelegate::DISCRIMINATOR],
    };

    (
        build_and_send_transaction(litesvm, &[payer], &payer.pubkey(), &[ix]),
        multi_delegate_pda,
        bump,
    )
}

pub fn create_simple_delegate_action(
    litesvm: &mut LiteSVM,
    payer: &Keypair,
    mint: Pubkey,
    delegate: Pubkey,
    amount: u64,
    expiry_s: u64,
    kind: TermsKind,
) -> (TransactionResult, Pubkey) {
    let (multi_delegate_pda, _bump) = get_multidelegate_pda(&payer.pubkey(), &mint);
    let (delegate_pda, _bump) =
        get_delegate_pda(&multi_delegate_pda, &delegate, &payer.pubkey(), kind);

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(multi_delegate_pda, false),
            AccountMeta::new(delegate_pda, false),
            AccountMeta::new_readonly(delegate, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: [
            vec![*add_delegate::DISCRIMINATOR],
            vec![kind as u8],
            amount.to_le_bytes().to_vec(),
            expiry_s.to_le_bytes().to_vec(),
        ]
        .concat(),
    };

    (
        build_and_send_transaction(litesvm, &[payer], &payer.pubkey(), &[ix]),
        delegate_pda,
    )
}
