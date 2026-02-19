pub mod subscription_cancelled;
pub mod subscription_created;

pub use subscription_cancelled::*;
pub use subscription_created::*;

pub enum SubscriptionEvent<'a> {
    Created(&'a SubscriptionCreatedEvent),
    Cancelled(&'a SubscriptionCancelledEvent),
}
