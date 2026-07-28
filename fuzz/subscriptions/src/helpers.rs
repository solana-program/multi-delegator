use std::rc::Rc;

use crucible_fuzzer::{AccountBuilderBase, TestContext};
use solana_clock::Clock;
use solana_keypair::Keypair;
use solana_program_pack::Pack;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;
#[cfg(feature = "invariant_subscriptions_hook")]
use spl_tlv_account_resolution::account::ExtraAccountMeta;
#[cfg(feature = "invariant_subscriptions_hook")]
use spl_tlv_account_resolution::state::ExtraAccountMetaList;
use spl_token_2022_interface::extension::transfer_hook::{TransferHook, TransferHookAccount};
use spl_token_2022_interface::extension::{BaseStateWithExtensionsMut, ExtensionType, StateWithExtensionsMut};
use spl_token_2022_interface::state::{Account, AccountState, Mint};
#[cfg(feature = "invariant_subscriptions_hook")]
use spl_transfer_hook_interface::instruction::ExecuteInstruction;
use subscriptions::accounts::Plan;
use subscriptions::SUBSCRIPTIONS_ID;

use crate::constants::{GENESIS_TS, INITIAL_LAMPORTS, SLOTS_PER_SECOND};

#[cfg(not(any(feature = "invariant_subscriptions_t22", feature = "invariant_subscriptions_hook")))]
pub const TOKEN_PROGRAM: Pubkey = spl_token_interface::ID;
#[cfg(any(feature = "invariant_subscriptions_t22", feature = "invariant_subscriptions_hook"))]
pub const TOKEN_PROGRAM: Pubkey = spl_token_2022_interface::ID;

pub const HOOK_PROGRAM: Pubkey = Pubkey::new_from_array([42u8; 32]);

// Mint / token-account extensions applied per feature. Empty for classic and plain Token-2022;
// the hook variant carries the paired TransferHook / TransferHookAccount extensions.
#[cfg(feature = "invariant_subscriptions_hook")]
const MINT_EXTENSIONS: &[ExtensionType] = &[ExtensionType::TransferHook];
#[cfg(not(feature = "invariant_subscriptions_hook"))]
const MINT_EXTENSIONS: &[ExtensionType] = &[];
#[cfg(feature = "invariant_subscriptions_hook")]
const ATA_EXTENSIONS: &[ExtensionType] = &[ExtensionType::TransferHookAccount];
#[cfg(not(feature = "invariant_subscriptions_hook"))]
const ATA_EXTENSIONS: &[ExtensionType] = &[];

pub fn set_clock(ctx: &mut TestContext, ts: i64) {
    let slot = ((ts - GENESIS_TS).max(0) * SLOTS_PER_SECOND) as u64 + 1;
    ctx.warp_to_slot(slot);
    ctx.set_sysvar(&Clock {
        slot,
        epoch_start_timestamp: GENESIS_TS,
        epoch: 0,
        leader_schedule_epoch: 0,
        unix_timestamp: ts,
    });
}

pub fn create_funded_wallet(ctx: &mut TestContext) -> Rc<Keypair> {
    let wallet = Rc::new(Keypair::new());
    ctx.create_account()
        .pubkey(wallet.pubkey())
        .lamports(INITIAL_LAMPORTS)
        .owner(solana_sdk_ids::system_program::ID)
        .create()
        .expect("create wallet");
    wallet
}

pub fn ata_address(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    get_associated_token_address_with_program_id(owner, mint, &TOKEN_PROGRAM)
}

// Reads the SPL token `amount` from its fixed base offset (bytes 64..72). Unlike
// TestContext::token_balance, this works for Token-2022 accounts carrying extensions, whose
// length exceeds the classic 165-byte layout the unpack there requires.
pub fn token_amount(ctx: &TestContext, ata: &Pubkey) -> u64 {
    match ctx.get_account(ata) {
        Ok(acc) if acc.data.len() >= 72 => u64::from_le_bytes(acc.data[64..72].try_into().unwrap()),
        _ => 0,
    }
}

// One mint/token-account builder for all variants: the base 82/165-byte layout when no
// extensions apply, otherwise the extension-packed layout. MINT_EXTENSIONS/ATA_EXTENSIONS select
// the shape per feature, so classic, plain Token-2022, and the hook variant share this path.
pub fn create_mint(ctx: &mut TestContext, mint: &Pubkey, authority: &Pubkey, decimals: u8) {
    let base = Mint {
        mint_authority: Some(*authority).into(),
        supply: 0,
        decimals,
        is_initialized: true,
        freeze_authority: None.into(),
    };
    let data = if MINT_EXTENSIONS.is_empty() {
        let mut data = vec![0u8; Mint::LEN];
        base.pack_into_slice(&mut data);
        data
    } else {
        let space = ExtensionType::try_calculate_account_len::<Mint>(MINT_EXTENSIONS).unwrap();
        let mut data = vec![0u8; space];
        let mut state = StateWithExtensionsMut::<Mint>::unpack_uninitialized(&mut data).unwrap();
        state.base = base;
        state.pack_base();
        state.init_account_type().unwrap();
        for ext in MINT_EXTENSIONS {
            match ext {
                ExtensionType::TransferHook => {
                    let hook = state.init_extension::<TransferHook>(true).unwrap();
                    hook.authority = Some(*authority).try_into().unwrap();
                    hook.program_id = Some(HOOK_PROGRAM).try_into().unwrap();
                }
                other => panic!("unsupported mint extension: {other:?}"),
            }
        }
        drop(state);
        data
    };
    ctx.create_account()
        .pubkey(*mint)
        .lamports(INITIAL_LAMPORTS)
        .owner(TOKEN_PROGRAM)
        .data(&data)
        .create()
        .expect("create mint");
}

pub fn create_ata(ctx: &mut TestContext, owner: &Pubkey, mint: &Pubkey, amount: u64) -> Pubkey {
    let ata = ata_address(owner, mint);
    let base = Account {
        mint: *mint,
        owner: *owner,
        amount,
        delegate: None.into(),
        state: AccountState::Initialized,
        is_native: None.into(),
        delegated_amount: 0,
        close_authority: None.into(),
    };
    let data = if ATA_EXTENSIONS.is_empty() {
        let mut data = vec![0u8; Account::LEN];
        base.pack_into_slice(&mut data);
        data
    } else {
        let space = ExtensionType::try_calculate_account_len::<Account>(ATA_EXTENSIONS).unwrap();
        let mut data = vec![0u8; space];
        let mut state = StateWithExtensionsMut::<Account>::unpack_uninitialized(&mut data).unwrap();
        state.base = base;
        state.pack_base();
        state.init_account_type().unwrap();
        for ext in ATA_EXTENSIONS {
            match ext {
                ExtensionType::TransferHookAccount => {
                    state.init_extension::<TransferHookAccount>(true).unwrap();
                }
                other => panic!("unsupported account extension: {other:?}"),
            }
        }
        drop(state);
        data
    };
    ctx.create_account()
        .pubkey(ata)
        .lamports(INITIAL_LAMPORTS)
        .owner(TOKEN_PROGRAM)
        .data(&data)
        .create()
        .expect("create ata");
    ata
}

// Loads the transfer-hook example program and installs its extra-account-metas list plus the
// counter account it increments on each transfer. Returns (validation_pda, counter).
#[cfg(feature = "invariant_subscriptions_hook")]
pub fn setup_hook(ctx: &mut TestContext, mint: &Pubkey) -> (Pubkey, Pubkey) {
    ctx.add_program(&HOOK_PROGRAM, "../../tests/transfer-hook-example/target/deploy/transfer_hook_example.so")
        .expect("load hook program");

    let (validation_pda, _) = Pubkey::find_program_address(&[b"extra-account-metas", mint.as_ref()], &HOOK_PROGRAM);
    let counter = Pubkey::new_unique();

    let meta = ExtraAccountMeta {
        discriminator: 0,
        address_config: counter.to_bytes(),
        is_signer: false.into(),
        is_writable: true.into(),
    };
    let mut validation_data = vec![0u8; ExtraAccountMetaList::size_of(1).unwrap()];
    ExtraAccountMetaList::init::<ExecuteInstruction>(&mut validation_data, &[meta]).unwrap();

    ctx.create_account()
        .pubkey(validation_pda)
        .lamports(INITIAL_LAMPORTS)
        .owner(HOOK_PROGRAM)
        .data(&validation_data)
        .create()
        .expect("create validation pda");
    ctx.create_account()
        .pubkey(counter)
        .lamports(INITIAL_LAMPORTS)
        .owner(HOOK_PROGRAM)
        .data(&[0u8])
        .create()
        .expect("create counter");

    (validation_pda, counter)
}

// Plan::find_pda in the generated client encodes plan_id as a string; the program derives
// the PDA with to_le_bytes.
pub fn plan_pda_address(owner: &Pubkey, plan_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[Plan::PREFIX, owner.as_ref(), &plan_id.to_le_bytes()], &SUBSCRIPTIONS_ID)
}

// FixedDelegation::find_pda in the generated client encodes nonce as a string; the program
// derives the PDA with to_le_bytes.
pub fn delegation_pda_address(authority: &Pubkey, delegator: &Pubkey, delegatee: &Pubkey, nonce: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"delegation", authority.as_ref(), delegator.as_ref(), delegatee.as_ref(), &nonce.to_le_bytes()],
        &SUBSCRIPTIONS_ID,
    )
}
