pub trait Observable {
    type Item;
    type Error;
}

pub trait IntoObservable {
    type ObservableType: Observable;

    fn into_observable(self) -> Self::ObservableType;
}

pub trait Observation {
    fn cancel(self);
}
