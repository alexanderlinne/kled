use crate::core;
use crate::flow;
use async_trait::async_trait;

#[operator(type = "flow", item = "ItemOut")]
pub struct Scan<ItemOut, BinaryOp>
where
    ItemOut: Clone,
    BinaryOp: FnMut(ItemOut, Item) -> ItemOut,
{
    initial_value: ItemOut,
    binary_op: BinaryOp,
}

#[derive(new)]
struct ScanSubscriber<Subscriber, ItemOut, BinaryOp> {
    subscriber: Subscriber,
    previous_value: ItemOut,
    binary_op: BinaryOp,
}

#[async_trait]
impl<Subscription, ItemIn, Subscriber, ItemOut, Error, BinaryOp>
    core::Subscriber<Subscription, ItemIn, Error> for ScanSubscriber<Subscriber, ItemOut, BinaryOp>
where
    Subscriber: core::Subscriber<Subscription, ItemOut, Error> + Send,
    Subscription: Send + 'static,
    BinaryOp: FnMut(ItemOut, ItemIn) -> ItemOut + Send,
    ItemIn: Send + 'static,
    ItemOut: Clone + Send,
    Error: Send + 'static,
{
    async fn on_subscribe(&mut self, cancellable: Subscription) {
        self.subscriber.on_subscribe(cancellable).await;
        let value = self.previous_value.clone();
        self.subscriber.on_next(value).await;
    }

    async fn on_next(&mut self, item: ItemIn) {
        self.previous_value = (self.binary_op)(self.previous_value.clone(), item);
        let value = self.previous_value.clone();
        self.subscriber.on_next(value).await;
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

    #[async_std::test]
    async fn scan() {
        vec![0, 1, 2, 3]
            .into_flow()
            .scan(0, |a, b| a + b)
            .into_step_verifier()
            .expect_subscription()
            .expect_all_of(vec![0, 0, 1, 3, 6])
            .expect_completed()
            .verify()
            .await;
    }
}
