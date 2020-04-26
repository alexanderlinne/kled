use crate::core;

pub trait Observable {
    type Item;
    type Error;

    fn subscribe<ObserverType>(self, observer: ObserverType)
    where
        ObserverType: core::Observer<Self::Item, Self::Error> + Send + Sync + 'static;
}

pub trait Subscription {
    fn unsubscribe(&mut self);
}

pub trait IntoObservable {
    type ObservableType: Observable;

    fn into_observable(self) -> Self::ObservableType;
}
