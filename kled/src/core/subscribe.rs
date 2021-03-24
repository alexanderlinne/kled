use crate::core;
use async_trait::async_trait;

#[async_trait]
pub trait ObservableSubsribeNext<NextFn, Cancellable, Item> {
    type Cancellable: core::Cancellable;

    async fn subscribe_next(self, _: NextFn) -> Self::Cancellable;
}

#[async_trait]
pub trait FlowSubsribeNext<NextFn, Subscription, Item> {
    type Subscription: core::Subscription;

    async fn subscribe_next(self, _: NextFn) -> Self::Subscription;
}

#[async_trait]
pub trait ObservableSubsribeAll<NextFn, ErrorFn, CompletedFn, Cancellable, Item, Error> {
    type Cancellable: core::Cancellable;

    async fn subscribe_all(self, _: NextFn, _: ErrorFn, _: CompletedFn) -> Self::Cancellable;
}

#[async_trait]
pub trait FlowSubsribeAll<NextFn, ErrorFn, CompletedFn, Subscription, Item, Error> {
    type Subscription: core::Subscription;

    async fn subscribe_all(self, _: NextFn, _: ErrorFn, _: CompletedFn) -> Self::Subscription;
}
