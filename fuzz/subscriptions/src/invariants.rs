use crucible_fuzzer::*;
use solana_signer::Signer;
use subscriptions::accounts::{RecurringDelegation, SubscriptionDelegation};

use crate::constants::{INITIAL_TOKENS, SUBSCRIBER_COUNT};
use crate::fixture::SubscriptionsFixture;
use crate::helpers::ata_address;

const RECURRING_NONCES: [u64; 3] = [3, 4, 5];

pub fn check_subscriptions_decodable_and_capped(fixture: &SubscriptionsFixture) {
    for subscriber in &fixture.subscribers {
        let subscription_pda = fixture.subscription_pda(&subscriber.pubkey());
        if !fixture.ctx.account_has_data(&subscription_pda, 1) {
            continue;
        }
        let account = fixture.ctx.get_account(&subscription_pda).expect("subscription account");
        let subscription = SubscriptionDelegation::from_bytes(&account.data);
        fuzz_assert!(subscription.is_ok(), "subscription account must stay decodable");
        let Ok(subscription) = subscription else { continue };
        fuzz_assert_le!(
            subscription.amount_pulled_in_period,
            subscription.terms.amount,
            "amount pulled in period exceeds authorized plan amount"
        );
    }
}

pub fn check_recurring_delegation_caps(fixture: &SubscriptionsFixture) {
    for subscriber in &fixture.subscribers {
        for nonce in RECURRING_NONCES {
            let pda = fixture.recurring_delegation_pda(&subscriber.pubkey(), nonce);
            if !fixture.ctx.account_has_data(&pda, 1) {
                continue;
            }
            let account = fixture.ctx.get_account(&pda).expect("recurring delegation account");
            let Ok(delegation) = RecurringDelegation::from_bytes(&account.data) else { continue };
            fuzz_assert_le!(
                delegation.amount_pulled_in_period,
                delegation.amount_per_period,
                "recurring delegation pulled more than the authorized per-period amount"
            );
        }
    }
}

pub fn check_dead_authority_inert(fixture: &mut SubscriptionsFixture) {
    for idx in 0..fixture.subscribers.len() {
        let subscriber = fixture.subscribers[idx].pubkey();
        let balance = fixture.ctx.token_balance(&ata_address(&subscriber, &fixture.mint));
        let alive = fixture.read_authority(&subscriber).is_some();
        let (prev_balance, prev_alive) = fixture.prev_spend_state[idx];
        if !prev_alive {
            fuzz_assert_ge!(
                balance,
                prev_balance,
                "tokens left a subscriber ATA while its subscription authority was closed"
            );
        }
        fixture.prev_spend_state[idx] = (balance, alive);
    }
}

pub fn check_token_conservation(fixture: &SubscriptionsFixture) {
    let subscriber_total: u64 =
        fixture.subscribers.iter().map(|s| fixture.ctx.token_balance(&ata_address(&s.pubkey(), &fixture.mint))).sum();
    let merchant_balance = fixture.ctx.token_balance(&fixture.merchant_ata);
    fuzz_assert_eq!(
        subscriber_total + merchant_balance,
        INITIAL_TOKENS * SUBSCRIBER_COUNT as u64,
        "token supply not conserved across pulls"
    );
}
