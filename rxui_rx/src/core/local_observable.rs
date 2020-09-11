use crate::core;
use crate::operators;

pub trait LocalObservable<'o>: core::Observable {
    type Cancellable: core::Cancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + 'o;

    fn scan<ItemOut, BinaryOp>(
        self,
        initial_value: ItemOut,
        binary_op: BinaryOp,
    ) -> operators::ObservableScan<Self, ItemOut, BinaryOp>
    where
        Self: Sized,
    {
        operators::ObservableScan::new(self, initial_value, binary_op)
    }
}
