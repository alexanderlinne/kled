use crate::core;

pub trait IntoObservable<Cancellable, Item, Error> {
    type Observable: core::Observable<Cancellable, Item, Error>;

    fn into_observable(self) -> Self::Observable;
}

pub trait IntoFlow<Subscription, Item, Error> {
    type Flow: core::Flow<Subscription, Item, Error>;

    fn into_flow(self) -> Self::Flow;
}

pub trait IntoObservableEmitter<Item, Error> {
    type Emitter: core::ObservableEmitter<Item, Error>;

    fn into_emitter(self) -> Self::Emitter;
}

pub trait IntoFlowEmitter<Item, Error> {
    type Emitter: core::FlowEmitter<Item, Error>;

    fn into_emitter(self) -> Self::Emitter;
}
