use std::rc::Rc;

use crucible_fuzzer::{AccountBuilderBase, TestContext};
use solana_clock::Clock;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;
use subscriptions::accounts::Plan;
use subscriptions::SUBSCRIPTIONS_ID;

use crate::constants::{GENESIS_TS, INITIAL_LAMPORTS, SLOTS_PER_SECOND};

#[cfg(not(feature = "invariant_subscriptions_t22"))]
pub const TOKEN_PROGRAM: Pubkey = spl_token_interface::ID;
#[cfg(feature = "invariant_subscriptions_t22")]
pub const TOKEN_PROGRAM: Pubkey = Pubkey::from_str_const("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

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

#[cfg(not(feature = "invariant_subscriptions_t22"))]
pub fn create_mint(ctx: &mut TestContext, mint: &Pubkey, authority: &Pubkey, decimals: u8) {
    ctx.create_mint().pubkey(*mint).mint_authority(*authority).decimals(decimals).create().expect("create mint");
}

#[cfg(not(feature = "invariant_subscriptions_t22"))]
pub fn create_ata(ctx: &mut TestContext, owner: &Pubkey, mint: &Pubkey, amount: u64) -> Pubkey {
    let ata = ata_address(owner, mint);
    ctx.create_token_account().pubkey(ata).mint(*mint).token_owner(*owner).amount(amount).create().expect("create ata");
    ata
}

// Token-2022 base accounts share the classic SPL layout (82-byte mint, 165-byte token account) and
// are accepted by the program without an account-type discriminator, so they are packed directly.
#[cfg(feature = "invariant_subscriptions_t22")]
pub fn create_mint(ctx: &mut TestContext, mint: &Pubkey, authority: &Pubkey, decimals: u8) {
    use solana_program_pack::Pack;
    use spl_token_interface::state::Mint;

    let mut data = vec![0u8; Mint::LEN];
    Mint {
        mint_authority: Some(*authority).into(),
        supply: 0,
        decimals,
        is_initialized: true,
        freeze_authority: None.into(),
    }
    .pack_into_slice(&mut data);
    ctx.create_account()
        .pubkey(*mint)
        .lamports(INITIAL_LAMPORTS)
        .owner(TOKEN_PROGRAM)
        .data(&data)
        .create()
        .expect("create token-2022 mint");
}

#[cfg(feature = "invariant_subscriptions_t22")]
pub fn create_ata(ctx: &mut TestContext, owner: &Pubkey, mint: &Pubkey, amount: u64) -> Pubkey {
    use solana_program_pack::Pack;
    use spl_token_interface::state::{Account, AccountState};

    let ata = ata_address(owner, mint);
    let mut data = vec![0u8; Account::LEN];
    Account {
        mint: *mint,
        owner: *owner,
        amount,
        delegate: None.into(),
        state: AccountState::Initialized,
        is_native: None.into(),
        delegated_amount: 0,
        close_authority: None.into(),
    }
    .pack_into_slice(&mut data);
    ctx.create_account()
        .pubkey(ata)
        .lamports(INITIAL_LAMPORTS)
        .owner(TOKEN_PROGRAM)
        .data(&data)
        .create()
        .expect("create token-2022 ata");
    ata
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
