use crate::cancellable::*;
use crate::core;
use async_trait::async_trait;

pub struct LambdaObserver<Cancellable, NextFn, ErrorFn, CompletedFn>
where
    Cancellable: core::Cancellable,
{
    stub: LazyCancellableStub<Cancellable>,
    item_consumer: NextFn,
    error_consumer: ErrorFn,
    completed_consumer: CompletedFn,
}

impl<Cancellable, NextFn, ErrorFn, CompletedFn>
    LambdaObserver<Cancellable, NextFn, ErrorFn, CompletedFn>
where
    Cancellable: core::Cancellable,
{
    pub fn new(
        item_consumer: NextFn,
        error_consumer: ErrorFn,
        completed_consumer: CompletedFn,
    ) -> Self {
        LambdaObserver {
            stub: LazyCancellableStub::default(),
            item_consumer,
            error_consumer,
            completed_consumer,
        }
    }

    pub fn cancellable(&self) -> LazyCancellable<Cancellable> {
        self.stub.cancellable()
    }
}

#[async_trait]
impl<Cancellable, NextFn, ErrorFn, CompletedFn, Item, Error>
    core::Observer<Cancellable, Item, Error>
    for LambdaObserver<Cancellable, NextFn, ErrorFn, CompletedFn>
where
    Cancellable: core::Cancellable + Send + Sync,
    Item: Send + 'static,
    Error: Send + 'static,
    NextFn: FnMut(Item) + Send,
    ErrorFn: FnMut(Error) + Send,
    CompletedFn: FnMut() + Send,
{
    async fn on_subscribe(&mut self, cancellable: Cancellable) {
        self.stub.set_cancellable(cancellable).await;
    }

    async fn on_next(&mut self, item: Item) {
        (self.item_consumer)(item)
    }

    async fn on_error(&mut self, error: Error) {
        (self.error_consumer)(error)
    }

    async fn on_completed(&mut self) {
        (self.completed_consumer)()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[async_std::test]
    async fn subscribe_next() {
        let mut expected = 0;
        vec![0, 1, 2, 3]
            .into_observable()
            .subscribe_next(move |item| {
                assert_eq!(item, expected);
                expected += 1;
            })
            .await;
    }
}
