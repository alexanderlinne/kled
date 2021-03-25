use crate::{core, flow, util};
use crate::flow::Signal;
use async_trait::async_trait;

#[operator(
    type = "flow",
    upstream_subscription = "util::Never",
    upstream_item = "Signal<Subscription, Item, Error>",
    upstream_error = "util::Never"
)]
pub struct Dematerialize {}

#[derive(new)]
struct DematerializeSubscriber<Subscriber> {
    subscriber: Subscriber,
}

#[async_trait]
impl<Subscription, Subscriber, Item, Error>
    core::Subscriber<util::Never, Signal<Subscription, Item, Error>, util::Never> for DematerializeSubscriber<Subscriber>
where
    Subscriber: core::Subscriber<Subscription, Item, Error> + Send,
    Subscription: Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
    async fn on_subscribe(&mut self, _: util::Never) {
        unreachable! {}
    }
    async fn on_next(&mut self, signal: Signal<Subscription, Item, Error>) {
        match signal {
            Signal::Subscribe(subscription) => {
                self.subscriber.on_subscribe(subscription).await;
            }
            Signal::Item(item) => {
                self.subscriber.on_next(item).await;
            },
            Signal::Error(err) => {
                self.subscriber.on_error(err).await;
            },
            Signal::Completed => {}
        }
    }
    async fn on_error(&mut self, _: flow::Error<util::Never>) {
        unreachable! {}
    }
    async fn on_completed(&mut self) {
        self.subscriber.on_completed().await;
    }
}
