use crate::core;

struct FnObserver<NextFn, ErrorFn, CompletedFn> {
    item_consumer: NextFn,
    error_consumer: ErrorFn,
    completed_consumer: CompletedFn,
}

impl<NextFn, ErrorFn, CompletedFn> FnObserver<NextFn, ErrorFn, CompletedFn> {
    pub fn new(
        item_consumer: NextFn,
        error_consumer: ErrorFn,
        completed_consumer: CompletedFn,
    ) -> Self {
        Self {
            item_consumer,
            error_consumer,
            completed_consumer,
        }
    }
}

impl<NextFn, ErrorFn, CompletedFn, Item, Error> core::Observer<Item, Error>
    for FnObserver<NextFn, ErrorFn, CompletedFn>
where
    NextFn: FnMut(Item) + Send + Sync + 'static,
    ErrorFn: FnMut(Error) + Send + Sync + 'static,
    CompletedFn: FnMut() + Send + Sync + 'static,
{
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

pub fn from_fn<NextFn, ErrorFn, CompletedFn, Item, Error>(
    item_consumer: NextFn,
    error_consumer: ErrorFn,
    completed_consumer: CompletedFn,
) -> impl core::Observer<Item, Error>
where
    NextFn: Fn(Item) + Send + Sync + 'static,
    ErrorFn: Fn(Error) + Send + Sync + 'static,
    CompletedFn: Fn() + Send + Sync + 'static,
{
    FnObserver::new(item_consumer, error_consumer, completed_consumer)
}

pub fn from_next_fn<NextFn, Item>(item_consumer: NextFn) -> impl core::Observer<Item, ()>
where
    NextFn: Fn(Item) + Send + Sync + 'static,
{
    FnObserver::new(item_consumer, |_| {}, || {})
}
