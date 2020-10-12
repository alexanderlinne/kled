use crate::core;
use crate::flow;
use std::marker::PhantomData;

#[derive(new)]
pub struct Map<Flow, Subscription, Item, Error, ItemOut, UnaryOp>
where
    Flow: core::Flow<Subscription, Item, Error>,
    UnaryOp: FnMut(Item) -> ItemOut,
{
    flow: Flow,
    unary_op: UnaryOp,
    phantom: PhantomData<(Subscription, Item, Error, ItemOut)>,
}

impl<Flow, Subscription, Item, Error, ItemOut, UnaryOp> core::Flow<Subscription, ItemOut, Error>
    for Map<Flow, Subscription, Item, Error, ItemOut, UnaryOp>
where
    Flow: core::Flow<Subscription, Item, Error>,
    Subscription: core::Subscription,
    UnaryOp: FnMut(Item) -> ItemOut + Send + 'static,
    ItemOut: Send + 'static,
{
    fn subscribe<Downstream>(self, downstream: Downstream)
    where
        Downstream: core::Subscriber<Subscription, ItemOut, Error> + Send + 'static,
    {
        self.flow
            .subscribe(MapSubscriber::new(downstream, self.unary_op));
    }
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
    use crate::subscriber::*;

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
