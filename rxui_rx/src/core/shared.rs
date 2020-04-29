use crate::core;

#[derive(Clone)]
pub struct Shared<Observable> {
    pub(crate) actual_observable: Observable,
}

impl<Observable> Shared<Observable> {
    pub fn new(actual_observable: Observable) -> Self {
        Self { actual_observable }
    }
}

impl<T> core::Observable for Shared<T>
where
    T: core::Observable,
{
    type Item = T::Item;
    type Error = T::Error;
}

impl<T> core::SharedObservable for Shared<T>
where
    T: core::SharedObservable,
{
    type Subscription = T::Subscription;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer:
            core::observer::Observer<T::Subscription, T::Item, T::Error> + Send + Sync + 'static,
    {
        self.actual_observable.actual_subscribe(observer)
    }
}

impl<SubscriptionIn, Item, Error, T> core::SharedSubject<SubscriptionIn, Item, Error> for Shared<T> where
    T: core::SharedSubject<SubscriptionIn, Item, Error>
{
}

impl<Subscription, Item, Error, T> core::Observer<Subscription, Item, Error> for Shared<T>
where
    T: core::Observer<Subscription, Item, Error>,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        self.actual_observable.on_subscribe(subscription);
    }

    fn on_next(&mut self, item: Item) {
        self.actual_observable.on_next(item);
    }

    fn on_error(&mut self, error: Error) {
        self.actual_observable.on_error(error);
    }

    fn on_completed(&mut self) {
        self.actual_observable.on_completed();
    }
}
