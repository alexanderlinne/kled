pub trait Observable {
    type Item;
    type Error;
}

pub trait ObservableSubscription {
    fn cancel(self);
}
