use crate::core;
use crate::flow;

#[derive(new, reactive_operator)]
pub struct FlowScan<Flow, ItemOut, BinaryOp>
where
    Flow: core::Flow,
    ItemOut: Clone,
    BinaryOp: FnMut(ItemOut, Flow::Item) -> ItemOut,
{
    #[upstream(downstream = "ScanSubscriber", item = "ItemOut")]
    flow: Flow,
    initial_value: ItemOut,
    binary_op: BinaryOp,
}

#[derive(new)]
struct ScanSubscriber<Subscriber, ItemOut, BinaryOp> {
    subscriber: Subscriber,
    previous_value: ItemOut,
    binary_op: BinaryOp,
}

impl<Subscription, ItemIn, Subscriber, ItemOut, Error, BinaryOp>
    core::Subscriber<Subscription, ItemIn, Error> for ScanSubscriber<Subscriber, ItemOut, BinaryOp>
where
    Subscriber: core::Subscriber<Subscription, ItemOut, Error>,
    BinaryOp: FnMut(ItemOut, ItemIn) -> ItemOut,
    ItemOut: Clone,
{
    fn on_subscribe(&mut self, cancellable: Subscription) {
        self.subscriber.on_subscribe(cancellable);
    }
    fn on_next(&mut self, item: ItemIn) {
        self.previous_value = (self.binary_op)(self.previous_value.clone(), item);
        self.subscriber.on_next(self.previous_value.clone());
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
            .scan(0, |a, b| a + b)
            .subscribe(test_subscriber.clone());

        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0, 1, 3, 6]);
    }
}
