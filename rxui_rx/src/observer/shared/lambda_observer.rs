use crate::cancellable::shared::*;
use crate::core;
use crate::marker;
use crate::util;

impl<'o, Observable, NextFn> core::ObservableSubsribeNext<'o, NextFn> for marker::Shared<Observable>
where
    Observable: core::SharedObservable + Send + 'static,
    Observable::Cancellable: Send + 'static,
    Observable::Error: util::Inconstructible,
    NextFn: FnMut(Observable::Item) + Send + 'static,
{
    type Cancellable = EitherCancellable<BoolCancellable, Observable::Cancellable>;

    fn subscribe_next(self, next_fn: NextFn) -> Self::Cancellable {
        use self::core::CancellableProvider;
        let observer = LambdaObserver::new(
            next_fn,
            |_| {
                panic! {}
            },
            || {},
        );
        let cancellable = observer.cancellable();
        self.actual.actual_subscribe(observer);
        cancellable
    }
}

impl<'o, Observable, NextFn, ErrorFn, CompletedFn>
    core::ObservableSubsribeAll<'o, NextFn, ErrorFn, CompletedFn> for marker::Shared<Observable>
where
    Observable: core::SharedObservable + Send + 'static,
    Observable::Cancellable: Send + 'static,
    NextFn: FnMut(Observable::Item) + Send + 'static,
    ErrorFn: FnMut(Observable::Error) + Send + 'static,
    CompletedFn: FnMut() + Send + 'static,
{
    type Cancellable = EitherCancellable<BoolCancellable, Observable::Cancellable>;

    fn subscribe_all(
        self,
        next_fn: NextFn,
        error_fn: ErrorFn,
        complete_fn: CompletedFn,
    ) -> Self::Cancellable {
        use crate::core::CancellableProvider;
        let observer = LambdaObserver::new(next_fn, error_fn, complete_fn);
        let cancellable = observer.cancellable();
        self.actual.actual_subscribe(observer);
        cancellable
    }
}

pub struct LambdaObserver<Cancellable, NextFn, ErrorFn, CompletedFn>
where
    Cancellable: core::Cancellable,
{
    cancellable: EitherCancellable<BoolCancellable, Cancellable>,
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
            cancellable: EitherCancellable::from_left(BoolCancellable::default()),
            item_consumer,
            error_consumer,
            completed_consumer,
        }
    }
}

impl<Cancellable, NextFn, ErrorFn, CompletedFn> core::CancellableProvider
    for LambdaObserver<Cancellable, NextFn, ErrorFn, CompletedFn>
where
    Cancellable: core::Cancellable,
{
    type Cancellable = EitherCancellable<BoolCancellable, Cancellable>;

    fn cancellable(&self) -> Self::Cancellable {
        self.cancellable.clone()
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
        use self::core::Cancellable;
        if self.cancellable.is_cancelled() {
            cancellable.cancel()
        }
        self.cancellable.set_right(cancellable);
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
