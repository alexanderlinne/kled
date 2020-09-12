use crate::core;

#[derive(new, reactive_operator)]
pub struct ObservableScan<Observable, ItemOut, BinaryOp>
where
    Observable: core::Observable,
    ItemOut: Clone,
    BinaryOp: FnMut(ItemOut, Observable::Item) -> ItemOut,
{
    #[upstream(downstream = "ScanObserver", item = "ItemOut")]
    observable: Observable,
    initial_value: ItemOut,
    binary_op: BinaryOp,
}

impl<'o, Observable, ItemOut, BinaryOp> core::LocalObservable<'o>
    for ObservableScan<Observable, ItemOut, BinaryOp>
where
    Observable: core::LocalObservable<'o>,
    ItemOut: Clone + 'o,
    BinaryOp: FnMut(ItemOut, Observable::Item) -> ItemOut + 'o,
{
    type Cancellable = Observable::Cancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + 'o,
    {
        self.observable.actual_subscribe(ScanObserver::new(
            observer,
            self.initial_value,
            self.binary_op,
        ));
    }
}

impl<Observable, ItemOut, BinaryOp> core::SharedObservable
    for ObservableScan<Observable, ItemOut, BinaryOp>
where
    Observable: core::SharedObservable,
    ItemOut: Clone + Send + 'static,
    BinaryOp: FnMut(ItemOut, Observable::Item) -> ItemOut + Send + 'static,
{
    type Cancellable = Observable::Cancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + Send + 'static,
    {
        self.observable.actual_subscribe(ScanObserver::new(
            observer,
            self.initial_value,
            self.binary_op,
        ));
    }
}

#[derive(new)]
struct ScanObserver<Observer, ItemOut, BinaryOp> {
    observer: Observer,
    previous_value: ItemOut,
    binary_op: BinaryOp,
}

impl<Cancellable, ItemIn, Observer, ItemOut, Error, BinaryOp>
    core::Observer<Cancellable, ItemIn, Error> for ScanObserver<Observer, ItemOut, BinaryOp>
where
    Observer: core::Observer<Cancellable, ItemOut, Error>,
    BinaryOp: FnMut(ItemOut, ItemIn) -> ItemOut,
    ItemOut: Clone,
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        self.observer.on_subscribe(cancellable);
    }
    fn on_next(&mut self, item: ItemIn) {
        self.previous_value = (self.binary_op)(self.previous_value.clone(), item);
        self.observer.on_next(self.previous_value.clone());
    }
    fn on_error(&mut self, error: Error) {
        self.observer.on_error(error);
    }
    fn on_completed(&mut self) {
        self.observer.on_completed();
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::util;

    #[test]
    fn local_scan() {
        use util::local::*;
        let test_observer = TestObserver::default();
        vec![0, 1, 2, 3]
            .into_observable()
            .scan(0, |a, b| a + b)
            .subscribe(test_observer.clone());

        assert_eq!(test_observer.status(), ObserverStatus::Completed);
        assert_eq!(test_observer.items(), vec![0, 1, 3, 6]);
    }
}
