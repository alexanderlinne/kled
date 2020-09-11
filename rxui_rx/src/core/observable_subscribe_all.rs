use crate::core;

pub trait ObservableSubsribeAll<'o, NextFn, ErrorFn, CompletedFn>: core::util::Sealed {
    type Cancellable: core::Cancellable;

    fn subscribe_all(self, _: NextFn, _: ErrorFn, _: CompletedFn) -> Self::Cancellable;
}

impl<'o, Observable, NextFn, ErrorFn, CompletedFn>
    ObservableSubsribeAll<'o, NextFn, ErrorFn, CompletedFn> for Observable
where
    Observable: core::LocalObservable<'o> + 'o,
    NextFn: FnMut(Observable::Item) + 'o,
    ErrorFn: FnMut(Observable::Error) + 'o,
    CompletedFn: FnMut() + 'o,
{
    type Cancellable = <LocalObserver<
        Observable::Cancellable,
        NextFn,
        ErrorFn,
        CompletedFn,
    > as core::CancellableProvider>::Cancellable;

    fn subscribe_all(
        self,
        next_fn: NextFn,
        error_fn: ErrorFn,
        complete_fn: CompletedFn,
    ) -> Self::Cancellable {
        use self::core::CancellableProvider;
        let observer = LocalObserver::new(next_fn, error_fn, complete_fn);
        let cancellable = observer.cancellable();
        self.actual_subscribe(observer);
        cancellable
    }
}

impl<'o, Observable, NextFn, ErrorFn, CompletedFn>
    ObservableSubsribeAll<'o, NextFn, ErrorFn, CompletedFn> for core::Shared<Observable>
where
    Observable: core::SharedObservable + Send + 'static,
    Observable::Cancellable: Send + 'static,
    NextFn: FnMut(Observable::Item) + Send + 'static,
    ErrorFn: FnMut(Observable::Error) + Send + 'static,
    CompletedFn: FnMut() + Send + 'static,
{
    type Cancellable = <SharedObserver<
        Observable::Cancellable,
        NextFn,
        ErrorFn,
        CompletedFn,
    > as core::CancellableProvider>::Cancellable;

    fn subscribe_all(
        self,
        next_fn: NextFn,
        error_fn: ErrorFn,
        complete_fn: CompletedFn,
    ) -> Self::Cancellable {
        use self::core::CancellableProvider;
        let observer = SharedObserver::new(next_fn, error_fn, complete_fn);
        let cancellable = observer.cancellable();
        self.actual_observable.actual_subscribe(observer);
        cancellable
    }
}

pub struct LocalObserver<Cancellable, NextFn, ErrorFn, CompletedFn>
where
    Cancellable: core::Cancellable,
{
    cancellable: core::LocalEitherCancellable<core::LocalCancellable, Cancellable>,
    item_consumer: NextFn,
    error_consumer: ErrorFn,
    completed_consumer: CompletedFn,
}

impl<Cancellable, NextFn, ErrorFn, CompletedFn>
    LocalObserver<Cancellable, NextFn, ErrorFn, CompletedFn>
where
    Cancellable: core::Cancellable,
{
    pub fn new(
        item_consumer: NextFn,
        error_consumer: ErrorFn,
        completed_consumer: CompletedFn,
    ) -> Self {
        LocalObserver {
            cancellable: core::LocalEitherCancellable::from_left(core::LocalCancellable::default()),
            item_consumer,
            error_consumer,
            completed_consumer,
        }
    }
}

impl<Cancellable, NextFn, ErrorFn, CompletedFn> core::CancellableProvider
    for LocalObserver<Cancellable, NextFn, ErrorFn, CompletedFn>
where
    Cancellable: core::Cancellable,
{
    type Cancellable = core::LocalEitherCancellable<core::LocalCancellable, Cancellable>;

    fn cancellable(&self) -> Self::Cancellable {
        self.cancellable.clone()
    }
}

impl<Cancellable, NextFn, ErrorFn, CompletedFn, Item, Error>
    core::Observer<Cancellable, Item, Error>
    for LocalObserver<Cancellable, NextFn, ErrorFn, CompletedFn>
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

pub struct SharedObserver<Cancellable, NextFn, ErrorFn, CompletedFn>
where
    Cancellable: core::Cancellable,
{
    cancellable: core::SharedEitherCancellable<core::SharedCancellable, Cancellable>,
    item_consumer: NextFn,
    error_consumer: ErrorFn,
    completed_consumer: CompletedFn,
}

impl<Cancellable, NextFn, ErrorFn, CompletedFn>
    SharedObserver<Cancellable, NextFn, ErrorFn, CompletedFn>
where
    Cancellable: core::Cancellable,
{
    pub fn new(
        item_consumer: NextFn,
        error_consumer: ErrorFn,
        completed_consumer: CompletedFn,
    ) -> Self {
        SharedObserver {
            cancellable: core::SharedEitherCancellable::from_left(
                core::SharedCancellable::default(),
            ),
            item_consumer,
            error_consumer,
            completed_consumer,
        }
    }
}

impl<Cancellable, NextFn, ErrorFn, CompletedFn> core::CancellableProvider
    for SharedObserver<Cancellable, NextFn, ErrorFn, CompletedFn>
where
    Cancellable: core::Cancellable,
{
    type Cancellable = core::SharedEitherCancellable<core::SharedCancellable, Cancellable>;

    fn cancellable(&self) -> Self::Cancellable {
        self.cancellable.clone()
    }
}

impl<Cancellable, NextFn, ErrorFn, CompletedFn, Item, Error>
    core::Observer<Cancellable, Item, Error>
    for SharedObserver<Cancellable, NextFn, ErrorFn, CompletedFn>
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
