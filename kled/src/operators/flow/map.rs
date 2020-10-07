use crate::core;
use crate::flow;
use std::marker::PhantomData;

#[derive(new, reactive_operator)]
pub struct FlowMap<Flow, ItemOut, UnaryOp>
where
    Flow: core::Flow,
    ItemOut: Clone,
    UnaryOp: FnMut(Flow::Item) -> ItemOut,
{
    #[upstream(item = "ItemOut")]
    flow: Flow,
    unary_op: UnaryOp,
}

#[derive(new)]
struct MapSubscriber<Subscriber, ItemOut, UnaryOp> {
    subscriber: Subscriber,
    unary_op: UnaryOp,
    phantom: PhantomData<ItemOut>,
}

impl<Subscription, ItemIn, Subscriber, ItemOut, Error, UnaryOp>
    core::Subscriber<Subscription, ItemIn, Error> for MapSubscriber<Subscriber, ItemOut, UnaryOp>
where
    Subscriber: core::Subscriber<Subscription, ItemOut, Error>,
    UnaryOp: FnMut(ItemIn) -> ItemOut,
    ItemOut: Clone,
{
    fn on_subscribe(&mut self, cancellable: Subscription) {
        self.subscriber.on_subscribe(cancellable);
    }
    fn on_next(&mut self, item: ItemIn) {
        self.subscriber.on_next((self.unary_op)(item));
    }
    fn on_error(&mut self, error: flow::Error<Error>) {
        self.subscriber.on_error(error);
    }
    fn on_completed(&mut self) {
        self.subscriber.on_completed();
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::subscriber::local::*;

    #[test]
    fn local_scan() {
        let test_subscriber = TestSubscriber::default();
        vec![0, 1, 2, 3]
            .into_flow()
            .map(|a| a + 1)
            .subscribe(test_subscriber.clone());

        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![1, 2, 3, 4]);
    }
}
