use crate::core;

pub trait IntoObservable {
    type Observable: core::Observable;

    fn into_observable(self) -> Self::Observable;
}

pub trait IntoSharedObservable {
    type Observable: core::SharedObservable;

    fn into_shared_observable(self) -> core::Shared<Self::Observable>;
}

impl<T> IntoSharedObservable for T
where
    T: IntoObservable,
    T::Observable: core::SharedObservable,
{
    type Observable = T::Observable;

    fn into_shared_observable(self) -> core::shared::Shared<Self::Observable> {
        use crate::core::SharedObservable;
        self.into_observable().into_shared()
    }
}
