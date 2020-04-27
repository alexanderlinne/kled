use crate::core;

pub struct FnObserver<SubscriptionFn, NextFn, ErrorFn, CompletedFn> {
    subscription_consumer: SubscriptionFn,
    item_consumer: NextFn,
    error_consumer: ErrorFn,
    completed_consumer: CompletedFn,
}

impl<SubscriptionFn, NextFn, ErrorFn, CompletedFn>
    FnObserver<SubscriptionFn, NextFn, ErrorFn, CompletedFn>
{
    pub fn new(
        subscription_consumer: SubscriptionFn,
        item_consumer: NextFn,
        error_consumer: ErrorFn,
        completed_consumer: CompletedFn,
    ) -> Self {
        Self {
            subscription_consumer,
            item_consumer,
            error_consumer,
            completed_consumer,
        }
    }
}

impl<SubscriptionFn, NextFn, ErrorFn, CompletedFn, Subscription, Item, Error>
    core::Observer<Subscription, Item, Error>
    for FnObserver<SubscriptionFn, NextFn, ErrorFn, CompletedFn>
where
    SubscriptionFn: FnMut(Subscription),
    NextFn: FnMut(Item),
    ErrorFn: FnMut(Error),
    CompletedFn: FnMut(),
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        (self.subscription_consumer)(subscription)
    }

    fn on_next(&mut self, item: Item) {
        (self.item_consumer)(item)
    }

    fn on_error(&mut self, error: Error) {
        (self.error_consumer)(error)
    }

    fn on_completed(&mut self) {
        (self.completed_consumer)()
    }
}

pub fn from_fn<SubscriptionFn, NextFn, ErrorFn, CompletedFn>(
    subscription_consumer: SubscriptionFn,
    item_consumer: NextFn,
    error_consumer: ErrorFn,
    completed_consumer: CompletedFn,
) -> FnObserver<SubscriptionFn, NextFn, ErrorFn, CompletedFn> {
    FnObserver::new(
        subscription_consumer,
        item_consumer,
        error_consumer,
        completed_consumer,
    )
}
