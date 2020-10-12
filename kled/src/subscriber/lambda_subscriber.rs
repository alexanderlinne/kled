use crate::core;
use crate::flow;
use crate::subscription::*;
use crate::util;

impl<'o, Flow, Subscription, Item, NextFn> core::FlowSubsribeNext<NextFn, Subscription, Item>
    for Flow
where
    Flow: core::Flow<Subscription, Item, util::Infallible> + Send + 'static,
    Subscription: core::Subscription + Send + Sync + 'static,
    Item: Send + 'static,
    Subscription: core::Subscription + Send + 'static,
    NextFn: FnMut(Item) + Send + 'static,
{
    type Subscription = LazySubscription<Subscription>;

    fn subscribe_next(self, next_fn: NextFn) -> Self::Subscription {
        let subscriber = LambdaSubscriber::new(
            next_fn,
            |_| {
                panic! {}
            },
            || {},
        );
        let subscription = subscriber.stub.subscription();
        self.subscribe(subscriber);
        subscription
    }
}

impl<Flow, NextFn, ErrorFn, CompletedFn, Subscription, Item, Error>
    core::FlowSubsribeAll<NextFn, ErrorFn, CompletedFn, Subscription, Item, Error> for Flow
where
    Flow: core::Flow<Subscription, Item, Error> + Send + 'static,
    Subscription: core::Subscription + Send + Sync + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    NextFn: FnMut(Item) + Send + 'static,
    ErrorFn: FnMut(flow::Error<Error>) + Send + 'static,
    CompletedFn: FnMut() + Send + 'static,
{
    type Subscription = LazySubscription<Subscription>;

    fn subscribe_all(
        self,
        next_fn: NextFn,
        error_fn: ErrorFn,
        complete_fn: CompletedFn,
    ) -> Self::Subscription {
        let subscriber = LambdaSubscriber::new(next_fn, error_fn, complete_fn);
        let subscription = subscriber.stub.subscription();
        self.subscribe(subscriber);
        subscription
    }
}

pub struct LambdaSubscriber<Subscription, NextFn, ErrorFn, CompletedFn>
where
    Subscription: core::Subscription,
{
    stub: LazySubscriptionStub<Subscription>,
    item_consumer: NextFn,
    error_consumer: ErrorFn,
    completed_consumer: CompletedFn,
}

impl<Subscription, NextFn, ErrorFn, CompletedFn>
    LambdaSubscriber<Subscription, NextFn, ErrorFn, CompletedFn>
where
    Subscription: core::Subscription,
{
    pub fn new(
        item_consumer: NextFn,
        error_consumer: ErrorFn,
        completed_consumer: CompletedFn,
    ) -> Self {
        LambdaSubscriber {
            stub: LazySubscriptionStub::default(),
            item_consumer,
            error_consumer,
            completed_consumer,
        }
    }
}

impl<Subscription, NextFn, ErrorFn, CompletedFn, Item, Error>
    core::Subscriber<Subscription, Item, Error>
    for LambdaSubscriber<Subscription, NextFn, ErrorFn, CompletedFn>
where
    Subscription: core::Subscription,
    NextFn: FnMut(Item),
    ErrorFn: FnMut(flow::Error<Error>),
    CompletedFn: FnMut(),
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        self.stub.set_subscription(subscription);
    }

    fn on_next(&mut self, item: Item) {
        (self.item_consumer)(item)
    }

    fn on_error(&mut self, error: flow::Error<Error>) {
        (self.error_consumer)(error)
    }

    fn on_completed(&mut self) {
        (self.completed_consumer)()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn subscribe_next() {
        let mut expected = 0;
        vec![0, 1, 2, 3].into_flow().subscribe_next(move |item| {
            assert_eq!(item, expected);
            expected += 1;
        });
    }
}
