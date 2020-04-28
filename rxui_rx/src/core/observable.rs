use crate::core;

pub trait Observable {
    type Item;
    type Error;
}

pub trait IntoObservable {
    type Observable: core::Observable;

    fn into_observable(self) -> Self::Observable;
}
