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
    get_associated_token_address_with_program_id(owner, mint, &spl_token_interface::ID)
}

pub fn create_ata(ctx: &mut TestContext, owner: &Pubkey, mint: &Pubkey, amount: u64) -> Pubkey {
    let ata = ata_address(owner, mint);
    ctx.create_token_account()
        .pubkey(ata)
        .mint(*mint)
        .token_owner(*owner)
        .amount(amount)
        .create()
        .expect("create token account");
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
