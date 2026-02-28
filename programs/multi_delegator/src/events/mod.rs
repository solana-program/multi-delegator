pub mod fixed_transfer;
pub mod recurring_transfer;
pub mod subscription_cancelled;
pub mod subscription_created;
pub mod subscription_transfer;

pub use fixed_transfer::*;
pub use recurring_transfer::*;
pub use subscription_cancelled::*;
pub use subscription_created::*;
pub use subscription_transfer::*;

pub enum Event<'a> {
    SubscriptionCreated(&'a SubscriptionCreatedEvent),
    SubscriptionCancelled(&'a SubscriptionCancelledEvent),
    SubscriptionTransfer(&'a SubscriptionTransferEvent),
    FixedTransfer(&'a FixedTransferEvent),
    RecurringTransfer(&'a RecurringTransferEvent),
}
