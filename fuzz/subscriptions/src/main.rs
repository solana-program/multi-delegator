mod constants;
mod fixture;
mod helpers;
mod invariants;

use crucible_fuzzer::*;

use crate::fixture::{SubscriptionsFixture, __subscriptions_fixture_fuzz};
use crate::invariants::{
    check_dead_authority_inert, check_recurring_delegation_caps, check_subscriptions_decodable_and_capped,
    check_token_conservation,
};

fn check_all(fixture: &mut SubscriptionsFixture) {
    check_subscriptions_decodable_and_capped(fixture);
    check_recurring_delegation_caps(fixture);
    check_token_conservation(fixture);
    check_dead_authority_inert(fixture);
}

#[invariant_test]
fn invariant_subscriptions(fixture: &mut SubscriptionsFixture) {
    check_all(fixture);
}

// Same actions and invariants against a Token-2022 mint; the token program is selected at
// build time by the matching feature (see helpers::TOKEN_PROGRAM).
#[invariant_test]
fn invariant_subscriptions_t22(fixture: &mut SubscriptionsFixture) {
    check_all(fixture);
}
