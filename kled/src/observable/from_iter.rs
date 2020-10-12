use crate::cancellable::*;
use crate::core;
use crate::observable;
use crate::util;

#[doc(hidden)]
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

impl<IntoIter> core::Observable<BoolCancellable, IntoIter::Item, util::Infallible>
    for IntoIterObservable<IntoIter>
where
    IntoIter: IntoIterator,
    IntoIter::Item: Send + 'static,
{
    fn subscribe<Observer>(self, observer: Observer)
    where
        Observer:
            core::Observer<BoolCancellable, IntoIter::Item, util::Infallible> + Send + 'static,
    {
        let mut observer = observable::Emitter::from(observer);
        for v in self.iterable.into_iter() {
            if !observer.is_cancelled() {
                observer.on_next(v);
            } else {
                break;
            }
        }
        if !observer.is_cancelled() {
            observer.on_completed();
        }
    }
}

impl<IntoIter> core::IntoObservable<BoolCancellable, IntoIter::Item, util::Infallible> for IntoIter
where
    IntoIter: IntoIterator,
    IntoIter::Item: Send + 'static,
{
    type Observable = IntoIterObservable<IntoIter>;

    fn into_observable(self) -> Self::Observable {
        IntoIterObservable::new(self)
    }
}
