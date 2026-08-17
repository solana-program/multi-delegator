//! Exercises the allowlist transfer-hook example: subscription pulls succeed
//! only when the destination owner has an allow-entry, direct owner transfers
//! bypass the allowlist, and foreign SPL delegates are rejected outright.

use crate::{
    state::plan::Plan,
    tests::{
        asserts::TransactionResultExt,
        constants::{MINT_DECIMALS, TOKEN_2022_PROGRAM_ID},
        utils::{
            allowlist_entry_pda, build_and_send_transaction, current_ts, days, get_ata_balance, init_ata, init_mint,
            initialize_subscription_authority_action, install_allowlist_entry, install_allowlist_hook_metas,
            load_transfer_hook_allowlist_example, set_transfer_hook_config, setup, CreatePlan, CreateSubscription,
            TransferSubscription, TRANSFER_HOOK_ALLOWLIST_PROGRAM_ID,
        },
    },
};
use litesvm::{types::TransactionResult, LiteSVM};
use solana_instruction::{error::InstructionError, AccountMeta};
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction_error::TransactionError;
use spl_token_2022_interface::extension::ExtensionType;
use spl_token_2022_interface::instruction::{approve_checked, transfer_checked};

#[allow(clippy::type_complexity)]
fn setup_allowlist_subscription(
    merchant_allowlisted: bool,
) -> (LiteSVM, Keypair, Keypair, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey) {
    let (mut litesvm, alice) = setup();
    load_transfer_hook_allowlist_example(&mut litesvm);
    let merchant = Keypair::new();
    litesvm.airdrop(&merchant.pubkey(), 10_000_000_000).unwrap();

    let mint = init_mint(
        &mut litesvm,
        TOKEN_2022_PROGRAM_ID,
        MINT_DECIMALS,
        1_000_000_000,
        Some(alice.pubkey()),
        &[ExtensionType::TransferHook],
    );
    set_transfer_hook_config(&mut litesvm, mint, Some(alice.pubkey()), Some(TRANSFER_HOOK_ALLOWLIST_PROGRAM_ID));
    let validation_pda = install_allowlist_hook_metas(&mut litesvm, &alice, mint);
    let allow_entry = allowlist_entry_pda(merchant.pubkey());
    if merchant_allowlisted {
        install_allowlist_entry(&mut litesvm, &alice, merchant.pubkey());
    }

    let alice_ata = init_ata(&mut litesvm, mint, alice.pubkey(), 100_000_000);
    let merchant_ata = init_ata(&mut litesvm, mint, merchant.pubkey(), 0);

    initialize_subscription_authority_action(&mut litesvm, &alice, mint).0.assert_ok();

    let (res, plan_pda) = CreatePlan::new(&mut litesvm, &merchant, mint)
        .plan_id(1)
        .amount(50_000_000)
        .period_hours(1)
        .end_ts(current_ts() + days(30) as i64)
        .execute();
    res.assert_ok();

    let svm_ts = litesvm.get_sysvar::<solana_clock::Clock>().unix_timestamp;
    let plan_terms = {
        let plan_account = litesvm.get_account(&plan_pda).unwrap();
        let plan = Plan::load(&plan_account.data).unwrap();
        plan.data.terms
    };
    let subscription_pda =
        CreateSubscription::new(&mut litesvm, plan_pda, alice.pubkey(), mint, svm_ts).terms(plan_terms).execute();

    (litesvm, alice, merchant, mint, plan_pda, subscription_pda, alice_ata, merchant_ata, validation_pda, allow_entry)
}

const HOOK_ERROR_DELEGATE_NOT_ALLOWED: u32 = 1;
const HOOK_ERROR_DESTINATION_NOT_ALLOWED: u32 = 2;

fn assert_hook_error(result: TransactionResult, expected_code: u32) {
    let err = result.expect_err("hook must reject the transfer").err;
    assert_eq!(err, TransactionError::InstructionError(0, InstructionError::Custom(expected_code)));
}

fn hook_remaining(validation_pda: Pubkey, allow_entry: Pubkey) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(TRANSFER_HOOK_ALLOWLIST_PROGRAM_ID, false),
        AccountMeta::new_readonly(validation_pda, false),
        AccountMeta::new_readonly(allow_entry, false),
    ]
}

#[test]
fn subscription_pull_to_allowlisted_destination_succeeds() {
    let (
        mut litesvm,
        alice,
        merchant,
        mint,
        plan_pda,
        subscription_pda,
        alice_ata,
        merchant_ata,
        validation_pda,
        allow_entry,
    ) = setup_allowlist_subscription(true);

    TransferSubscription::new(&mut litesvm, &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(10_000_000)
        .to(merchant_ata)
        .remaining(hook_remaining(validation_pda, allow_entry))
        .execute()
        .assert_ok();

    assert_eq!(get_ata_balance(&litesvm, &alice_ata), 90_000_000);
    assert_eq!(get_ata_balance(&litesvm, &merchant_ata), 10_000_000);
}

#[test]
fn subscription_pull_to_non_allowlisted_destination_fails() {
    let (
        mut litesvm,
        alice,
        merchant,
        mint,
        plan_pda,
        subscription_pda,
        _alice_ata,
        merchant_ata,
        validation_pda,
        allow_entry,
    ) = setup_allowlist_subscription(false);

    let result = TransferSubscription::new(&mut litesvm, &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(10_000_000)
        .to(merchant_ata)
        .remaining(hook_remaining(validation_pda, allow_entry))
        .execute();
    assert_hook_error(result, HOOK_ERROR_DESTINATION_NOT_ALLOWED);
    assert_eq!(get_ata_balance(&litesvm, &merchant_ata), 0);
}

#[test]
fn owner_direct_transfer_bypasses_allowlist() {
    let (
        mut litesvm,
        alice,
        _merchant,
        mint,
        _plan_pda,
        _subscription_pda,
        alice_ata,
        merchant_ata,
        validation_pda,
        allow_entry,
    ) = setup_allowlist_subscription(false);

    let mut ix = transfer_checked(
        &TOKEN_2022_PROGRAM_ID,
        &alice_ata,
        &mint,
        &merchant_ata,
        &alice.pubkey(),
        &[],
        5_000_000,
        MINT_DECIMALS,
    )
    .unwrap();
    ix.accounts.extend(hook_remaining(validation_pda, allow_entry));

    build_and_send_transaction(&mut litesvm, &[&alice], &alice.pubkey(), &ix).unwrap();
    assert_eq!(get_ata_balance(&litesvm, &merchant_ata), 5_000_000);
}

#[test]
fn foreign_delegate_rejected_even_to_allowlisted_destination() {
    let (
        mut litesvm,
        alice,
        _merchant,
        mint,
        _plan_pda,
        _subscription_pda,
        alice_ata,
        merchant_ata,
        validation_pda,
        allow_entry,
    ) = setup_allowlist_subscription(true);

    let bob = Keypair::new();
    litesvm.airdrop(&bob.pubkey(), 1_000_000_000).unwrap();

    let approve_ix = approve_checked(
        &TOKEN_2022_PROGRAM_ID,
        &alice_ata,
        &mint,
        &bob.pubkey(),
        &alice.pubkey(),
        &[],
        50_000_000,
        MINT_DECIMALS,
    )
    .unwrap();
    build_and_send_transaction(&mut litesvm, &[&alice], &alice.pubkey(), &approve_ix).unwrap();

    let mut ix = transfer_checked(
        &TOKEN_2022_PROGRAM_ID,
        &alice_ata,
        &mint,
        &merchant_ata,
        &bob.pubkey(),
        &[],
        5_000_000,
        MINT_DECIMALS,
    )
    .unwrap();
    ix.accounts.extend(hook_remaining(validation_pda, allow_entry));

    let result = build_and_send_transaction(&mut litesvm, &[&bob], &bob.pubkey(), &ix);
    assert_hook_error(result, HOOK_ERROR_DELEGATE_NOT_ALLOWED);
    assert_eq!(get_ata_balance(&litesvm, &merchant_ata), 0);
}
