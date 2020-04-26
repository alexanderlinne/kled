use crate::core;

struct FnObserver<SubscriptionFn, NextFn, ErrorFn, CompletedFn> {
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

impl<SubscriptionFn, NextFn, ErrorFn, CompletedFn, Item, Error> core::Observer<Item, Error>
    for FnObserver<SubscriptionFn, NextFn, ErrorFn, CompletedFn>
where
    SubscriptionFn: FnMut(Box<dyn core::observable::Subscription>) + Send + Sync + 'static,
    NextFn: FnMut(Item) + Send + Sync + 'static,
    ErrorFn: FnMut(Error) + Send + Sync + 'static,
    CompletedFn: FnMut() + Send + Sync + 'static,
{
    fn on_subscribe(&mut self, subscription: Box<dyn core::observable::Subscription>) {
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

pub fn from_fn<SubscriptionFn, NextFn, ErrorFn, CompletedFn, Item, Error>(
    subscription_consumer: SubscriptionFn,
    item_consumer: NextFn,
    error_consumer: ErrorFn,
    completed_consumer: CompletedFn,
) -> impl core::Observer<Item, Error>
where
    SubscriptionFn: FnMut(Box<dyn core::observable::Subscription>) + Send + Sync + 'static,
    NextFn: Fn(Item) + Send + Sync + 'static,
    ErrorFn: Fn(Error) + Send + Sync + 'static,
    CompletedFn: Fn() + Send + Sync + 'static,
{
    FnObserver::new(
        subscription_consumer,
        item_consumer,
        error_consumer,
        completed_consumer,
    )
}

pub fn from_next_fn<NextFn, Item>(
    item_consumer: NextFn,
) -> impl core::Observer<Item, ()>
where
    NextFn: Fn(Item) + Send + Sync + 'static,
{
    FnObserver::new(|_| {}, item_consumer, |_| {}, || {})
}
