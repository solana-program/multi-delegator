use pinocchio::error::ProgramError;

use crate::event_engine::{
    EventDiscriminators, EventSerialize, EVENT_DISCRIMINATOR_LEN, EVENT_IX_TAG_LE,
};
use crate::events::{SubscriptionCancelledEvent, SubscriptionCreatedEvent, SubscriptionEvent};
use crate::MultiDelegatorError;

pub fn decode_event<'a>(data: &'a [u8]) -> Result<SubscriptionEvent<'a>, ProgramError> {
    if data.len() < EVENT_DISCRIMINATOR_LEN {
        return Err(MultiDelegatorError::InvalidEventTag.into());
    }

    if data[..EVENT_IX_TAG_LE.len()] != EVENT_IX_TAG_LE {
        return Err(MultiDelegatorError::InvalidEventTag.into());
    }

    let discriminator = data[EVENT_IX_TAG_LE.len()];
    let payload = &data[EVENT_DISCRIMINATOR_LEN..];

    let disc = EventDiscriminators::try_from(discriminator)
        .map_err(|_| ProgramError::from(MultiDelegatorError::InvalidEventDiscriminator))?;

    match disc {
        EventDiscriminators::SubscriptionCreated => Ok(SubscriptionEvent::Created(
            SubscriptionCreatedEvent::load(payload)?,
        )),
        EventDiscriminators::SubscriptionCancelled => Ok(SubscriptionEvent::Cancelled(
            SubscriptionCancelledEvent::load(payload)?,
        )),
    }
}
