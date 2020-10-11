use crate::core;

pub trait ObservableSubsribeNext<NextFn, Cancellable, Item> {
    type Cancellable: core::Cancellable;

    fn subscribe_next(self, _: NextFn) -> Self::Cancellable;
}

pub trait FlowSubsribeNext<NextFn, Subscription, Item> {
    type Subscription: core::Subscription;

    fn subscribe_next(self, _: NextFn) -> Self::Subscription;
}

pub trait ObservableSubsribeAll<NextFn, ErrorFn, CompletedFn, Cancellable, Item, Error> {
    type Cancellable: core::Cancellable;

    fn subscribe_all(self, _: NextFn, _: ErrorFn, _: CompletedFn) -> Self::Cancellable;
}

pub trait FlowSubsribeAll<NextFn, ErrorFn, CompletedFn, Subscription, Item, Error> {
    type Subscription: core::Subscription;

    fn subscribe_all(self, _: NextFn, _: ErrorFn, _: CompletedFn) -> Self::Subscription;
}
