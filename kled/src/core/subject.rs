use crate::core;

pub trait Subject<SubscriptionIn, Item, Error>:
    core::Observable + core::Observer<SubscriptionIn, Item, Error>
{
}
