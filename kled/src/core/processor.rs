use crate::core;

pub trait LocalProcessor<'o, SubscriptionIn, Item, Error>:
    core::LocalFlow<'o> + core::Subscriber<SubscriptionIn, Item, Error>
{
}

pub trait SharedProcessor<SubscriptionIn, Item, Error>:
    core::SharedFlow + core::Subscriber<SubscriptionIn, Item, Error>
{
}
