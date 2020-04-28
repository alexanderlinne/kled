use crate::core;

pub trait IntoObservable {
    type Observable: core::Observable;

    fn into_observable(self) -> Self::Observable;
}
