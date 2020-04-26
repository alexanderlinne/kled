use crate::core;
use crate::core::{Observer, Subscriber};
use crate::subscriber::UnsubscribeMemorizingSubscriber;

pub struct IntoIterObservable<IntoIter> {
    iterable: IntoIter,
}

impl<IntoIter> IntoIterObservable<IntoIter>
where
    IntoIter: IntoIterator,
{
    fn new(iterable: IntoIter) -> Self {
        Self { iterable }
    }
}

impl<IntoIter> core::Observable for IntoIterObservable<IntoIter>
where
    IntoIter: IntoIterator,
{
    type Item = IntoIter::Item;
    type Error = ();

    fn subscribe<ObserverType>(self, observer: ObserverType)
    where
        ObserverType: core::Observer<Self::Item, Self::Error> + Send + Sync + 'static,
    {
        let mut subscriber = UnsubscribeMemorizingSubscriber::new(observer);
        subscriber.on_subscribe(Box::new(subscriber.create_subscription()));
        for v in self.iterable.into_iter() {
            if !subscriber.is_unsubscribed() {
                subscriber.on_next(v);
            }
        }
        subscriber.on_completed();
    }
}

impl<IntoIter> core::IntoObservable for IntoIter
where
    IntoIter: IntoIterator,
{
    type ObservableType = IntoIterObservable<IntoIter>;

    fn into_observable(self) -> <Self as core::observable::IntoObservable>::ObservableType {
        IntoIterObservable::new(self)
    }
}

#[cfg(test)]
mod test {
    use crate::observer;
    use crate::prelude::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn iterable_into_observable() {
        let vec = vec![0, 1, 2, 3];
        let sum = Arc::new(Mutex::new(0));
        let sum_move = sum.clone();
        vec.into_observable()
            .subscribe(observer::from_next_fn(move |v| {
                (*sum_move.lock().unwrap()) += v
            }));
        assert_eq!((*sum.lock().unwrap()), 6);
    }
}
