use crate::cancellable::*;
use crate::core;
use crate::util;

impl<'o, Observable, NextFn> core::ObservableSubsribeNext<NextFn> for Observable
where
    Observable: core::Observable + Send + 'static,
    Observable::Cancellable: Send + 'static,
    Observable::Error: util::Inconstructible,
    NextFn: FnMut(Observable::Item) + Send + 'static,
{
    type Cancellable = LazyCancellable<Observable::Cancellable>;

    fn subscribe_next(self, next_fn: NextFn) -> Self::Cancellable {
        use self::core::CancellableProvider;
        let observer = LambdaObserver::new(
            next_fn,
            |_| {
                panic! {}
            },
            || {},
        );
        let cancellable = observer.stub.cancellable();
        self.actual_subscribe(observer);
        cancellable
    }
}

impl<Observable, NextFn, ErrorFn, CompletedFn>
    core::ObservableSubsribeAll<NextFn, ErrorFn, CompletedFn> for Observable
where
    Observable: core::Observable + Send + 'static,
    Observable::Cancellable: Send + 'static,
    NextFn: FnMut(Observable::Item) + Send + 'static,
    ErrorFn: FnMut(Observable::Error) + Send + 'static,
    CompletedFn: FnMut() + Send + 'static,
{
    type Cancellable = LazyCancellable<Observable::Cancellable>;

    fn subscribe_all(
        self,
        next_fn: NextFn,
        error_fn: ErrorFn,
        complete_fn: CompletedFn,
    ) -> Self::Cancellable {
        use crate::core::CancellableProvider;
        let observer = LambdaObserver::new(next_fn, error_fn, complete_fn);
        let cancellable = observer.stub.cancellable();
        self.actual_subscribe(observer);
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

impl<Cancellable, NextFn, ErrorFn, CompletedFn, Item, Error>
    core::Observer<Cancellable, Item, Error>
    for LambdaObserver<Cancellable, NextFn, ErrorFn, CompletedFn>
where
    Cancellable: core::Cancellable,
    NextFn: FnMut(Item),
    ErrorFn: FnMut(Error),
    CompletedFn: FnMut(),
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        self.stub.set_cancellable(cancellable);
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn subscribe_next() {
        let mut expected = 0;
        vec![0, 1, 2, 3]
            .into_observable()
            .subscribe_next(move |item| {
                assert_eq!(item, expected);
                expected += 1;
            });
    }
}
