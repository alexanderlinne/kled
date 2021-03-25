use crate::{core, util};
use crate::observable::Signal;
use async_trait::async_trait;

#[operator(
    type = "observable",
    subscription = "util::Never",
    item = "Signal<Cancellable, Item, Error>",
    error = "util::Never"
)]
pub struct Materialize {}

#[derive(new)]
struct MaterializeObserver<Observer> {
    observer: Observer,
}

#[async_trait]
impl<Cancellable, Observer, Item, Error>
    core::Observer<Cancellable, Item, Error> for MaterializeObserver<Observer>
where
    Observer: core::Observer<util::Never, Signal<Cancellable, Item, Error>, util::Never> + Send,
    Cancellable: Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
    async fn on_subscribe(&mut self, cancellable: Cancellable) {
        self.observer.on_next(Signal::Subscribe(cancellable)).await;
    }
    async fn on_next(&mut self, item: Item) {
        self.observer.on_next(Signal::Item(item)).await;
    }
    async fn on_error(&mut self, error: Error) {
        self.observer.on_next(Signal::Error(error)).await;
    }
    async fn on_completed(&mut self) {
        self.observer.on_next(Signal::Completed).await;
        self.observer.on_completed().await;
    }
}
