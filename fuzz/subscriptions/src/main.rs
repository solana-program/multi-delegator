mod constants;
mod fixture;
mod helpers;
mod invariants;

use crucible_fuzzer::*;

use crate::fixture::{SubscriptionsFixture, __subscriptions_fixture_fuzz};
use crate::invariants::check_subscriptions_decodable_and_capped;

#[invariant_test]
fn invariant_subscriptions(fixture: &mut SubscriptionsFixture) {
    check_subscriptions_decodable_and_capped(fixture);
}
