use crucible_fuzzer::*;
use solana_signer::Signer;
use subscriptions::accounts::SubscriptionDelegation;

use crate::constants::{INITIAL_TOKENS, SUBSCRIBER_COUNT};
use crate::fixture::SubscriptionsFixture;
use crate::helpers::ata_address;

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
