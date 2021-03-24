use crate::core;
use crate::flow;
use async_trait::async_trait;
use std::marker::PhantomData;

#[operator(type = "flow", item = "ItemOut")]
pub struct Map<ItemOut, UnaryOp>
where
    UnaryOp: FnMut(Item) -> ItemOut + Send
{
    unary_op: UnaryOp,
}

#[derive(new)]
struct MapSubscriber<Subscriber, ItemOut, UnaryOp> {
    subscriber: Subscriber,
    unary_op: UnaryOp,
    phantom: PhantomData<ItemOut>,
}

#[async_trait]
impl<Subscription, ItemIn, Subscriber, ItemOut, Error, UnaryOp>
    core::Subscriber<Subscription, ItemIn, Error> for MapSubscriber<Subscriber, ItemOut, UnaryOp>
where
    Subscriber: core::Subscriber<Subscription, ItemOut, Error> + Send,
    Subscription: Send + 'static,
    ItemIn: Send + 'static,
    ItemOut: Send + 'static,
    Error: Send + 'static,
    UnaryOp: FnMut(ItemIn) -> ItemOut + Send,
{
    async fn on_subscribe(&mut self, cancellable: Subscription) {
        self.subscriber.on_subscribe(cancellable).await;
    }
    async fn on_next(&mut self, item: ItemIn) {
        self.subscriber.on_next((self.unary_op)(item)).await;
    }
    async fn on_error(&mut self, error: flow::Error<Error>) {
        self.subscriber.on_error(error).await;
    }
    async fn on_completed(&mut self) {
        self.subscriber.on_completed().await;
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::subscriber::*;

    #[async_std::test]
    async fn local_scan() {
        let test_subscriber = TestSubscriber::default();
        vec![0, 1, 2, 3]
            .into_flow()
            .map(|a| a + 1)
            .subscribe(test_subscriber.clone()).await;

        assert_eq!(test_subscriber.status().await, SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items().await, vec![1, 2, 3, 4]);
    }
}
