use crate::core;

pub trait ObservableSubsribeNext<'o, NextFn>: core::util::Sealed {
    type Cancellable: core::Cancellable;

    fn subscribe_next(self, _: NextFn) -> Self::Cancellable;
}

impl<'o, Observable, NextFn> ObservableSubsribeNext<'o, NextFn> for Observable
where
    Observable: core::LocalObservable<'o> + 'o,
    Observable::Error: core::Inconstructible,
    NextFn: FnMut(Observable::Item) + 'o,
{
    type Cancellable =
        <LocalObserver<Observable::Cancellable, NextFn> as core::CancellableProvider>::Cancellable;

    fn subscribe_next(self, next_fn: NextFn) -> Self::Cancellable {
        use self::core::CancellableProvider;
        let observer = LocalObserver::new(next_fn);
        let cancellable = observer.cancellable();
        self.actual_subscribe(observer);
        cancellable
    }
}

impl<'o, Observable, NextFn> ObservableSubsribeNext<'o, NextFn> for core::Shared<Observable>
where
    Observable: core::SharedObservable + Send + 'static,
    Observable::Cancellable: Send + 'static,
    Observable::Error: core::Inconstructible,
    NextFn: FnMut(Observable::Item) + Send + 'static,
{
    type Cancellable =
        <SharedObserver<Observable::Cancellable, NextFn> as core::CancellableProvider>::Cancellable;

    fn subscribe_next(self, next_fn: NextFn) -> Self::Cancellable {
        use self::core::CancellableProvider;
        let observer = SharedObserver::new(next_fn);
        let cancellable = observer.cancellable();
        self.actual_observable.actual_subscribe(observer);
        cancellable
    }
}

pub struct LocalObserver<Cancellable, NextFn>
where
    Cancellable: core::Cancellable,
{
    cancellable: core::LocalEitherCancellable<core::LocalCancellable, Cancellable>,
    item_consumer: NextFn,
}

impl<Cancellable, NextFn> LocalObserver<Cancellable, NextFn>
where
    Cancellable: core::Cancellable,
{
    pub fn new(item_consumer: NextFn) -> Self {
        LocalObserver {
            cancellable: core::LocalEitherCancellable::from_left(core::LocalCancellable::default()),
            item_consumer,
        }
    }
}

impl<Cancellable, NextFn> core::CancellableProvider for LocalObserver<Cancellable, NextFn>
where
    Cancellable: core::Cancellable,
{
    type Cancellable = core::LocalEitherCancellable<core::LocalCancellable, Cancellable>;

    fn cancellable(&self) -> Self::Cancellable {
        self.cancellable.clone()
    }
}

impl<Cancellable, NextFn, Item, Error> core::Observer<Cancellable, Item, Error>
    for LocalObserver<Cancellable, NextFn>
where
    Cancellable: core::Cancellable,
    Error: core::Inconstructible,
    NextFn: FnMut(Item),
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

    fn on_error(&mut self, _: Error) {
        panic!();
    }

    fn on_completed(&mut self) {}
}

pub struct SharedObserver<Cancellable, NextFn>
where
    Cancellable: core::Cancellable,
{
    cancellable: core::SharedEitherCancellable<core::SharedCancellable, Cancellable>,
    item_consumer: NextFn,
}

impl<Cancellable, NextFn> SharedObserver<Cancellable, NextFn>
where
    Cancellable: core::Cancellable,
{
    pub fn new(item_consumer: NextFn) -> Self {
        SharedObserver {
            cancellable: core::SharedEitherCancellable::from_left(
                core::SharedCancellable::default(),
            ),
            item_consumer,
        }
    }
}

impl<Cancellable, NextFn> core::CancellableProvider for SharedObserver<Cancellable, NextFn>
where
    Cancellable: core::Cancellable,
{
    type Cancellable = core::SharedEitherCancellable<core::SharedCancellable, Cancellable>;

    fn cancellable(&self) -> Self::Cancellable {
        self.cancellable.clone()
    }
}

impl<Cancellable, NextFn, Item, Error> core::Observer<Cancellable, Item, Error>
    for SharedObserver<Cancellable, NextFn>
where
    Cancellable: core::Cancellable,
    Error: core::Inconstructible,
    NextFn: FnMut(Item),
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

    fn on_error(&mut self, _: Error) {
        panic!();
    }

    fn on_completed(&mut self) {}
}
