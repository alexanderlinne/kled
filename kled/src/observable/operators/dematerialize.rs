use crate::observable::Signal;
use crate::{core, Never};
use async_trait::async_trait;

#[operator(
    type = "observable",
    upstream_subscription = "Never",
    upstream_item = "Signal<Cancellable, Item, Error>",
    upstream_error = "Never"
)]
pub struct Dematerialize {}

#[derive(new)]
struct DematerializeObserver<Observer> {
    observer: Observer,
}

#[async_trait]
impl<Cancellable, Observer, Item, Error>
    core::Observer<Never, Signal<Cancellable, Item, Error>, Never>
    for DematerializeObserver<Observer>
where
    Observer: core::Observer<Cancellable, Item, Error> + Send,
    Cancellable: Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
    async fn on_subscribe(&mut self, _: Never) {
        unreachable! {}
    }
    async fn on_next(&mut self, signal: Signal<Cancellable, Item, Error>) {
        match signal {
            Signal::Subscribe(cancellable) => {
                self.observer.on_subscribe(cancellable).await;
            }
            Signal::Item(item) => {
                self.observer.on_next(item).await;
            }
            Signal::Error(err) => {
                self.observer.on_error(err).await;
            }
            Signal::Completed => {}
        }
    }
    async fn on_error(&mut self, _: Never) {
        unreachable! {}
    }
    async fn on_completed(&mut self) {
        self.observer.on_completed().await;
    }
}
