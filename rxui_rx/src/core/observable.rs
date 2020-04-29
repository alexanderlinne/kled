use crate::operators;

pub trait Observable {
    type Item;
    type Error;

    fn scan<ItemOut, BinaryOp>(
        self,
        initial_value: ItemOut,
        binary_op: BinaryOp,
    ) -> operators::Scan<Self, ItemOut, BinaryOp>
    where
        Self: Sized,
    {
        operators::Scan::new(self, initial_value, binary_op)
    }
}

pub trait ObservableSubscription {
    fn cancel(self);
}
