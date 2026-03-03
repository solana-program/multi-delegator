//! Subscription delegation account.

use codama::CodamaAccount;
use core::mem::{size_of, transmute};
use pinocchio::error::ProgramError;

use crate::{
    state::common::AccountDiscriminator, state::header::DISCRIMINATOR_OFFSET, Header,
    MultiDelegatorError,
};

/// A subscriber's delegation linked to a specific [`Plan`](super::plan::Plan).
///
/// Created when a user subscribes to a plan. The plan owner (or whitelisted
/// pullers) can transfer up to the plan's `amount` per period from the
/// subscriber's ATA. Cancellation sets [`expires_at_ts`](Self::expires_at_ts)
/// to the end of the current billing period; after that timestamp, no further
/// transfers are allowed and the account can be closed via revoke.
///
/// **PDA seeds:** `["subscription", plan_pda, subscriber]`
#[repr(C, packed)]
#[derive(Debug, CodamaAccount)]
pub struct SubscriptionDelegation {
    /// Common delegation header (discriminator, version, bump, delegator, delegatee, payer).
    ///
    /// In this context, `delegator` is the subscriber and `delegatee` is the plan PDA.
    pub header: Header,
    /// Token amount already transferred in the current billing period.
    pub amount_pulled_in_period: u64,
    /// Unix timestamp marking the start of the current billing period.
    pub current_period_start_ts: i64,
    /// Unix timestamp after which this subscription is no longer valid.
    ///
    /// `0` means the subscription is active (not cancelled). Set to a future
    /// timestamp when the subscriber cancels; transfers are blocked once the
    /// clock passes this value.
    pub expires_at_ts: i64,
}

impl SubscriptionDelegation {
    /// Total serialized size in bytes.
    pub const LEN: usize = size_of::<Self>();

    /// PDA seed prefix.
    pub const SEED: &'static [u8] = b"subscription";

    /// Deserializes an immutable reference from raw account data.
    ///
    /// Returns an error if the data length or discriminator does not match.
    pub fn load(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        if bytes[DISCRIMINATOR_OFFSET] != AccountDiscriminator::SubscriptionDelegation as u8 {
            return Err(MultiDelegatorError::InvalidAccountDiscriminator.into());
        }
        Ok(unsafe { &*transmute::<*const u8, *const Self>(bytes.as_ptr()) })
    }

    /// Deserializes a mutable reference from raw account data.
    ///
    /// Returns an error if the data length or discriminator does not match.
    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(MultiDelegatorError::InvalidAccountData.into());
        }
        if bytes[DISCRIMINATOR_OFFSET] != AccountDiscriminator::SubscriptionDelegation as u8 {
            return Err(MultiDelegatorError::InvalidAccountDiscriminator.into());
        }
        Ok(unsafe { &mut *transmute::<*mut u8, *mut Self>(bytes.as_mut_ptr()) })
    }
}
