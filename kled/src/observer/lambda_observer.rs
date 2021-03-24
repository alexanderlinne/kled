use crate::cancellable::*;
use crate::core;
use crate::util;
use async_trait::async_trait;

#[async_trait]
impl<'o, Observable, NextFn, Cancellable, Item>
    core::ObservableSubsribeNext<NextFn, Cancellable, Item> for Observable
where
    Observable: core::Observable<Cancellable, Item, util::Infallible> + Send + 'static,
    Cancellable: core::Cancellable + Send + Sync + 'static,
    Item: Send + 'static,
    NextFn: FnMut(Item) + Send + 'static,
{
    type Cancellable = LazyCancellable<Cancellable>;

    async fn subscribe_next(self, next_fn: NextFn) -> Self::Cancellable {
        let observer = LambdaObserver::new(
            next_fn,
            |_| {
                panic! {}
            },
            || {},
        );
        let cancellable = observer.stub.cancellable();
        self.subscribe(observer).await;
        cancellable
    }
}

#[async_trait]
impl<Observable, NextFn, ErrorFn, CompletedFn, Cancellable, Item, Error>
    core::ObservableSubsribeAll<NextFn, ErrorFn, CompletedFn, Cancellable, Item, Error>
    for Observable
where
    Observable: core::Observable<Cancellable, Item, Error> + Send + 'static,
    Cancellable: core::Cancellable + Send + Sync + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    NextFn: FnMut(Item) + Send + 'static,
    ErrorFn: FnMut(Error) + Send + 'static,
    CompletedFn: FnMut() + Send + 'static,
{
    type Cancellable = LazyCancellable<Cancellable>;

    async fn subscribe_all(
        self,
        next_fn: NextFn,
        error_fn: ErrorFn,
        complete_fn: CompletedFn,
    ) -> Self::Cancellable {
        let observer = LambdaObserver::new(next_fn, error_fn, complete_fn);
        let cancellable = observer.stub.cancellable();
        self.subscribe(observer).await;
        cancellable
    }
}

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
