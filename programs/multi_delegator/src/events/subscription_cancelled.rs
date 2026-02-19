use core::mem::size_of;

use pinocchio::Address;

use crate::event_engine::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[repr(C, packed)]
pub struct SubscriptionCancelledEvent {
    pub plan: Address,
    pub subscriber: Address,
    pub revoked_ts: i64,
}

impl SubscriptionCancelledEvent {
    pub const DATA_LEN: usize = size_of::<Self>();

    pub fn new(plan: Address, subscriber: Address, revoked_ts: i64) -> Self {
        Self {
            plan,
            subscriber,
            revoked_ts,
        }
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
        writer.extend_from_slice(&{ self.revoked_ts }.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_engine::EVENT_IX_TAG_LE;
    use crate::events::SubscriptionEvent;
    use crate::tests::events::decode_event;

    fn plan() -> Address {
        Address::new_from_array([1u8; 32])
    }

    fn subscriber() -> Address {
        Address::new_from_array([2u8; 32])
    }

    #[test]
    fn roundtrip() {
        let event = SubscriptionCancelledEvent::new(plan(), subscriber(), 1_700_000_000);
        let bytes = event.to_bytes();
        let decoded = decode_event(&bytes).unwrap();

        match decoded {
            SubscriptionEvent::Cancelled(e) => {
                assert_eq!(e.plan, plan());
                assert_eq!(e.subscriber, subscriber());
                assert_eq!({ e.revoked_ts }, 1_700_000_000);
            }
            _ => panic!("expected Cancelled event"),
        }
    }

    #[test]
    fn wire_format() {
        let event = SubscriptionCancelledEvent::new(plan(), subscriber(), 99);
        let bytes = event.to_bytes();

        assert_eq!(&bytes[..8], &EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], SubscriptionCancelledEvent::DISCRIMINATOR);
        assert_eq!(&bytes[9..41], plan().as_ref());
        assert_eq!(&bytes[41..73], subscriber().as_ref());
        assert_eq!(&bytes[73..81], &99i64.to_le_bytes());
    }

    #[test]
    fn negative_timestamp() {
        let event = SubscriptionCancelledEvent::new(plan(), subscriber(), -1);
        let bytes = event.to_bytes();
        let decoded = decode_event(&bytes).unwrap();
        match decoded {
            SubscriptionEvent::Cancelled(e) => assert_eq!({ e.revoked_ts }, -1),
            _ => panic!("expected Cancelled"),
        }
    }
}
