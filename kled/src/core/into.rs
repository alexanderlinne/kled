use crate::core;

pub trait IntoObservable<Cancellable, Item, Error>
where
    Cancellable: core::Cancellable + Send + Sync + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
    type Observable: core::Observable<Cancellable, Item, Error>;

    fn into_observable(self) -> Self::Observable;
}

pub trait IntoFlow<Subscription, Item, Error>
where
    Subscription: core::Subscription + Send + Sync + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
    type Flow: core::Flow<Subscription, Item, Error>;

    fn into_flow(self) -> Self::Flow;
}
