use crate::core;

pub struct FnObserver<CancellableFn, NextFn, ErrorFn, CompletedFn> {
    cancellable_consumer: CancellableFn,
    item_consumer: NextFn,
    error_consumer: ErrorFn,
    completed_consumer: CompletedFn,
}

impl<CancellableFn, NextFn, ErrorFn, CompletedFn>
    FnObserver<CancellableFn, NextFn, ErrorFn, CompletedFn>
{
    pub fn new(
        cancellable_consumer: CancellableFn,
        item_consumer: NextFn,
        error_consumer: ErrorFn,
        completed_consumer: CompletedFn,
    ) -> Self {
        Self {
            cancellable_consumer,
            item_consumer,
            error_consumer,
            completed_consumer,
        }
    }
}

impl<CancellableFn, NextFn, ErrorFn, CompletedFn, Cancellable, Item, Error>
    core::Observer<Cancellable, Item, Error>
    for FnObserver<CancellableFn, NextFn, ErrorFn, CompletedFn>
where
    CancellableFn: FnMut(Cancellable),
    NextFn: FnMut(Item),
    ErrorFn: FnMut(Error),
    CompletedFn: FnMut(),
{
    fn on_subscribe(&mut self, subscription: Cancellable) {
        (self.cancellable_consumer)(subscription)
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

pub fn from_fn<CancellableFn, NextFn, ErrorFn, CompletedFn>(
    cancellable_consumer: CancellableFn,
    item_consumer: NextFn,
    error_consumer: ErrorFn,
    completed_consumer: CompletedFn,
) -> FnObserver<CancellableFn, NextFn, ErrorFn, CompletedFn> {
    FnObserver::new(
        cancellable_consumer,
        item_consumer,
        error_consumer,
        completed_consumer,
    )
}
