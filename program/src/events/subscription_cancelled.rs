use core::mem::size_of;

use alloc::vec::Vec;
use codama::CodamaEvent;
use pinocchio::Address;

use crate::event_engine::{EventDiscriminator, EventDiscriminators, EventSerialize};

/// Emitted when a subscription is cancelled.
#[repr(C, packed)]
#[derive(CodamaEvent)]
// EVENT_IX_TAG_LE @0, EventDiscriminators::SubscriptionCancelled @8
#[codama(discriminator(bytes = [228, 69, 165, 46, 81, 203, 154, 29], offset = 0))]
#[codama(discriminator(bytes = [1], offset = 8))]
pub struct SubscriptionCancelledEvent {
    /// The plan PDA the subscription belongs to.
    pub plan: Address,
    /// The subscriber's wallet address.
    pub subscriber: Address,
    /// Unix timestamp when the subscription will expire.
    pub expires_at_ts: i64,
    /// The address whose approval cancelled the subscription: the subscriber for
    /// `cancel_subscription`, the plan owner or a whitelisted puller for
    /// `cancel_subscription_now`.
    pub authorized_by: Address,
}

impl SubscriptionCancelledEvent {
    /// Wire-format payload size (excluding tag and discriminator).
    pub const DATA_LEN: usize = size_of::<Self>();

    /// Constructs a new event.
    pub fn new(plan: Address, subscriber: Address, expires_at_ts: i64, authorized_by: Address) -> Self {
        Self { plan, subscriber, expires_at_ts, authorized_by }
    }
}

impl EventDiscriminator for SubscriptionCancelledEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::SubscriptionCancelled as u8;
}

impl EventSerialize for SubscriptionCancelledEvent {
    const DATA_LEN: usize = Self::DATA_LEN;

    fn write_inner(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(self.plan.as_ref());
        writer.extend_from_slice(self.subscriber.as_ref());
        writer.extend_from_slice(&{ self.expires_at_ts }.to_le_bytes());
        writer.extend_from_slice(self.authorized_by.as_ref());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_engine::EVENT_IX_TAG_LE;
    use crate::events::Event;
    use crate::tests::events::decode_event;

    fn plan() -> Address {
        Address::new_from_array([1u8; 32])
    }

    fn subscriber() -> Address {
        Address::new_from_array([2u8; 32])
    }

    fn authorized_by() -> Address {
        Address::new_from_array([3u8; 32])
    }

    #[test]
    fn roundtrip() {
        let event = SubscriptionCancelledEvent::new(plan(), subscriber(), 1_700_000_000, authorized_by());
        let bytes = event.to_bytes();
        let decoded = decode_event(&bytes).unwrap();

        match decoded {
            Event::SubscriptionCancelled(e) => {
                assert_eq!(e.plan, plan());
                assert_eq!(e.subscriber, subscriber());
                assert_eq!({ e.expires_at_ts }, 1_700_000_000);
                assert_eq!(e.authorized_by, authorized_by());
            }
            _ => panic!("expected Cancelled event"),
        }
    }

    #[test]
    fn wire_format() {
        let event = SubscriptionCancelledEvent::new(plan(), subscriber(), 99, authorized_by());
        let bytes = event.to_bytes();

        assert_eq!(&bytes[..8], &EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], SubscriptionCancelledEvent::DISCRIMINATOR);
        assert_eq!(&bytes[9..41], plan().as_ref());
        assert_eq!(&bytes[41..73], subscriber().as_ref());
        assert_eq!(&bytes[73..81], &99i64.to_le_bytes());
        assert_eq!(&bytes[81..113], authorized_by().as_ref());
    }

    #[test]
    fn negative_timestamp() {
        let event = SubscriptionCancelledEvent::new(plan(), subscriber(), -1, authorized_by());
        let bytes = event.to_bytes();
        let decoded = decode_event(&bytes).unwrap();
        match decoded {
            Event::SubscriptionCancelled(e) => assert_eq!({ e.expires_at_ts }, -1),
            _ => panic!("expected Cancelled"),
        }
    }
}
