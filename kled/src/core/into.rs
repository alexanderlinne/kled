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

pub trait IntoFlow<Subscription, Item, Error> {
    type Flow: core::Flow<Subscription, Item, Error>;

    fn into_flow(self) -> Self::Flow;
}

pub trait IntoFlowEmitter<Item, Error> {
    type Emitter: core::FlowEmitter<Item, Error>;

    fn into_emitter(self) -> Self::Emitter;
}
