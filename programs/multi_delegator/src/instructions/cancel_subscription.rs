use pinocchio::{
    error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    AccountView, ProgramResult,
};

use crate::{
    check_and_update_version,
    event_engine::{self, EventSerialize},
    events::SubscriptionCancelledEvent,
    state::{plan::Plan, subscription_delegation::SubscriptionDelegation},
    AccountCheck, MultiDelegatorError, ProgramAccount, SignerAccount, WritableAccount,
};

/// Instruction discriminator byte for `CancelSubscription`.
pub const DISCRIMINATOR: &u8 = &12;

/// Cancels a subscription by setting its `expires_at_ts` to the end of the
/// current billing period.
///
/// After cancellation the subscription remains valid until `expires_at_ts`,
/// then it can be closed via [`RevokeDelegation`](crate::instructions::revoke_delegation).
/// Emits a [`SubscriptionCancelledEvent`].
pub fn process(accounts: &[AccountView]) -> ProgramResult {
    let accounts_struct = CancelSubscriptionAccounts::try_from(accounts)?;
    let current_ts = Clock::get()?.unix_timestamp;

    let expires_at_ts;
    let plan_pda;
    {
        let mut binding = accounts_struct.subscription_pda.try_borrow_mut()?;
        check_and_update_version(&mut binding)?;
        let subscription = SubscriptionDelegation::load_mut_with_min_size(&mut binding)?;

        // Verify caller is the subscriber (delegator)
        if subscription.header.delegator != *accounts_struct.subscriber.address() {
            return Err(MultiDelegatorError::Unauthorized.into());
        }

        // Check not already cancelled
        if subscription.expires_at_ts != 0 {
            return Err(MultiDelegatorError::SubscriptionAlreadyCancelled.into());
        }

        // Validate subscription's delegatee matches the passed plan_pda
        if subscription.header.delegatee != *accounts_struct.plan_pda.address() {
            return Err(MultiDelegatorError::SubscriptionPlanMismatch.into());
        }

        plan_pda = subscription.header.delegatee;

        // Compute expires_at_ts based on plan state
        if accounts_struct.plan_pda.owned_by(&crate::ID) {
            // Plan is valid — load it and verify terms match
            let plan_data = accounts_struct.plan_pda.try_borrow()?;
            let plan = Plan::load(&plan_data)?;

            if subscription.check_plan_terms(&plan.data.terms).is_err() {
                // Plan terms mismatch (ghost plan) — expire immediately
                expires_at_ts = current_ts;
            } else {
                // Terms match — compute end of current period
                let period_length_s =
                    (subscription.terms.period_hours as i64)
                        .checked_mul(3600)
                        .ok_or::<ProgramError>(MultiDelegatorError::ArithmeticOverflow.into())?;
                let period_start = subscription.current_period_start_ts;
                let elapsed = current_ts.saturating_sub(period_start);
                let periods_elapsed = elapsed / period_length_s;
                expires_at_ts = periods_elapsed
                    .checked_add(1)
                    .and_then(|p| p.checked_mul(period_length_s))
                    .and_then(|offset| period_start.checked_add(offset))
                    .ok_or::<ProgramError>(MultiDelegatorError::ArithmeticOverflow.into())?;
            }
        } else {
            // Plan is closed (not owned by our program) — expire immediately
            expires_at_ts = current_ts;
        }

        subscription.expires_at_ts = expires_at_ts;
    }

    // Emit SubscriptionCancelled event via self-CPI
    let event = SubscriptionCancelledEvent::new(
        plan_pda,
        *accounts_struct.subscriber.address(),
        expires_at_ts,
    );
    let event_data = event.to_bytes();
    event_engine::emit_event(
        &crate::ID,
        accounts_struct.event_authority,
        accounts_struct.self_program,
        &event_data,
    )?;

    Ok(())
}

/// Validated accounts for the [`CancelSubscription`](crate::MultiDelegatorInstruction::CancelSubscription) instruction.
pub struct CancelSubscriptionAccounts<'a> {
    pub subscriber: &'a AccountView,
    pub plan_pda: &'a AccountView,
    pub subscription_pda: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub self_program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for CancelSubscriptionAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [subscriber, plan_pda, subscription_pda, event_authority, self_program] = accounts
        else {
            return Err(MultiDelegatorError::NotEnoughAccountKeys.into());
        };

        SignerAccount::check(subscriber)?;
        ProgramAccount::check(subscription_pda)?;
        WritableAccount::check(subscription_pda)?;

        Ok(Self {
            subscriber,
            plan_pda,
            subscription_pda,
            event_authority,
            self_program,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        state::subscription_delegation::SubscriptionDelegation,
        tests::{
            asserts::TransactionResultExt,
            utils::{init_wallet, setup_with_subscription, CancelSubscription},
        },
        MultiDelegatorError,
    };
    #[test]
    fn cancel_subscription_happy_path() {
        let (mut litesvm, alice, _merchant, _mint, plan_pda, _plan_bump, subscription_pda) =
            setup_with_subscription();

        CancelSubscription::new(&mut litesvm, &alice, plan_pda, subscription_pda)
            .execute()
            .assert_ok();

        // Verify expires_at_ts is set (end of current period)
        let sub_account = litesvm.get_account(&subscription_pda).unwrap();
        let sub = SubscriptionDelegation::load(&sub_account.data).unwrap();
        assert_ne!({ sub.expires_at_ts }, 0);
    }

    #[test]
    fn cancel_subscription_non_subscriber_rejected() {
        let (mut litesvm, _alice, _merchant, _mint, plan_pda, _plan_bump, subscription_pda) =
            setup_with_subscription();

        let attacker = init_wallet(&mut litesvm, 10_000_000_000);
        let res =
            CancelSubscription::new(&mut litesvm, &attacker, plan_pda, subscription_pda).execute();
        res.assert_err(MultiDelegatorError::Unauthorized);
    }

    #[test]
    fn cancel_subscription_already_cancelled_rejected() {
        let (mut litesvm, alice, _merchant, _mint, plan_pda, _plan_bump, subscription_pda) =
            setup_with_subscription();

        // Cancel once
        CancelSubscription::new(&mut litesvm, &alice, plan_pda, subscription_pda)
            .execute()
            .assert_ok();

        // Cancel again should fail
        let res =
            CancelSubscription::new(&mut litesvm, &alice, plan_pda, subscription_pda).execute();
        res.assert_err(MultiDelegatorError::SubscriptionAlreadyCancelled);
    }

    #[test]
    fn test_cancel_subscription_version_mismatch() {
        use crate::state::header::VERSION_OFFSET;

        let (mut litesvm, alice, _merchant, _mint, plan_pda, _plan_bump, subscription_pda) =
            setup_with_subscription();

        let mut account = litesvm.get_account(&subscription_pda).unwrap();
        account.data[VERSION_OFFSET] = 0;
        litesvm.set_account(subscription_pda, account).unwrap();

        CancelSubscription::new(&mut litesvm, &alice, plan_pda, subscription_pda)
            .execute()
            .assert_err(MultiDelegatorError::MigrationRequired);
    }
}
