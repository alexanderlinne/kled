use crate::core;
use crate::marker;
use std::marker::PhantomData;

#[derive(new)]
pub struct Map<Observable, Cancellable, Item, Error, ItemOut, UnaryOp>
where
    Observable: marker::Observable<Cancellable, Item, Error>,
{
    observable: Observable,
    unary_op: UnaryOp,
    phantom: PhantomData<(Cancellable, Item, Error, ItemOut)>,
}

impl<Observable, Cancellable, Item, Error, ItemOut, UnaryOp>
    core::Observable<Cancellable, ItemOut, Error>
    for Map<Observable, Cancellable, Item, Error, ItemOut, UnaryOp>
where
    Observable: core::Observable<Cancellable, Item, Error>,
    Cancellable: core::Cancellable + Send + Sync + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    ItemOut: Send + 'static,
    UnaryOp: FnMut(Item) -> ItemOut + Send + 'static,
{
    fn subscribe<Downstream>(self, downstream: Downstream)
    where
        Downstream: core::Observer<Cancellable, ItemOut, Error> + Send + 'static,
    {
        self.observable
            .subscribe(MapObserver::new(downstream, self.unary_op));
    }
}

#[derive(new)]
struct MapObserver<Observer, ItemOut, UnaryOp> {
    observer: Observer,
    unary_op: UnaryOp,
    phantom: PhantomData<ItemOut>,
}

impl<Cancellable, ItemIn, Observer, ItemOut, Error, UnaryOp>
    core::Observer<Cancellable, ItemIn, Error> for MapObserver<Observer, ItemOut, UnaryOp>
where
    Observer: core::Observer<Cancellable, ItemOut, Error>,
    UnaryOp: FnMut(ItemIn) -> ItemOut,
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        self.observer.on_subscribe(cancellable);
    }
    fn on_next(&mut self, item: ItemIn) {
        self.observer.on_next((self.unary_op)(item));
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
    use crate::observer::*;
    use crate::prelude::*;

    #[test]
    fn local() {
        let test_observer = TestObserver::default();
        vec![0, 1, 2, 3]
            .into_observable()
            .map(|a| a + 1)
            .subscribe(test_observer.clone());

        assert_eq!(test_observer.status(), ObserverStatus::Completed);
        assert_eq!(test_observer.items(), vec![1, 2, 3, 4]);
    }
}
