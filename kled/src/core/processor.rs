use crate::core;

pub trait Processor<SubscriptionIn, Item, Error>:
    core::Flow + core::Subscriber<SubscriptionIn, Item, Error>
{
}
