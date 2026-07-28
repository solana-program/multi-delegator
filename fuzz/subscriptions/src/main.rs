mod constants;
mod fixture;
mod helpers;
mod invariants;

use crucible_fuzzer::*;

use crate::fixture::{SubscriptionsFixture, __subscriptions_fixture_fuzz};
use crate::invariants::{
    check_dead_authority_inert, check_fixed_delegation_cap, check_recurring_delegation_caps,
    check_subscriptions_decodable_and_capped, check_token_conservation,
};

fn check_all(fixture: &mut SubscriptionsFixture) {
    check_subscriptions_decodable_and_capped(fixture);
    check_recurring_delegation_caps(fixture);
    check_fixed_delegation_cap(fixture);
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

// Against a Token-2022 mint with a TransferHook extension: transfers forward the hook program
// and its extra accounts, exercising the program's hook-forwarding path. The hook only bumps a
// counter, so token conservation still holds and all invariants apply unchanged.
#[invariant_test]
fn invariant_subscriptions_hook(fixture: &mut SubscriptionsFixture) {
    check_all(fixture);
}
