use crate::core;

pub trait IntoObservable {
    type Observable: core::Observable;

    fn into_observable(self) -> Self::Observable;
}

pub trait IntoFlow {
    type Flow: core::Flow;

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
