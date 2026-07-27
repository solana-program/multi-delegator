use crucible_fuzzer::*;
use solana_signer::Signer;
use subscriptions::accounts::SubscriptionDelegation;

use crate::fixture::SubscriptionsFixture;

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
