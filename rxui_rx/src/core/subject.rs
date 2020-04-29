use crate::core;

pub trait LocalSubject<'o, SubscriptionIn, Item, Error>:
    core::LocalObservable<'o> + core::Observer<SubscriptionIn, Item, Error>
{
}

pub trait SharedSubject<SubscriptionIn, Item, Error>:
    core::SharedObservable + core::Observer<SubscriptionIn, Item, Error>
{
}
