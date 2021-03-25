use crate::cancellable::*;
use crate::core;
use crate::observable;
use crate::Never;
use async_trait::async_trait;

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

#[async_trait]
impl<IntoIter> core::Observable<ArcCancellable, IntoIter::Item, Never>
    for IntoIterObservable<IntoIter>
where
    IntoIter: IntoIterator + Send,
    IntoIter::Item: Send + 'static,
    IntoIter::IntoIter: Send,
{
    async fn subscribe<Observer>(self, observer: Observer)
    where
        Observer:
            core::Observer<ArcCancellable, IntoIter::Item, Never> + Send + 'static,
    {
        let mut observer = observable::Emitter::from(observer).await;
        for v in self.iterable.into_iter() {
            if !observer.is_cancelled() {
                observer.on_next(v).await;
            } else {
                break;
            }
        }
        if !observer.is_cancelled() {
            observer.on_completed().await;
        }
    }
}

impl<IntoIter> core::IntoObservable<ArcCancellable, IntoIter::Item, Never> for IntoIter
where
    IntoIter: IntoIterator + Send,
    IntoIter::Item: Send + 'static,
    IntoIter::IntoIter: Send,
{
    type Observable = IntoIterObservable<IntoIter>;

    fn into_observable(self) -> Self::Observable {
        IntoIterObservable::new(self)
    }
}
