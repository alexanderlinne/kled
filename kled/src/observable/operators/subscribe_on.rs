use crate::core;
use async_trait::async_trait;
use std::marker::PhantomData;

#[derive(new)]
pub struct SubscribeOn<Observable, Cancellable, Item, Error, Scheduler> {
    observable: Observable,
    scheduler: Scheduler,
    phantom: PhantomData<(Cancellable, Item, Error)>,
}

#[async_trait]
impl<Observable, Cancellable, Item, Error, Scheduler> core::Observable<Cancellable, Item, Error>
    for SubscribeOn<Observable, Cancellable, Item, Error, Scheduler>
where
    Observable: core::Observable<Cancellable, Item, Error> + Send + 'static,
    Cancellable: core::Cancellable + Send + Sync + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    Scheduler: core::Scheduler + Send + 'static,
{
    async fn subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Cancellable, Item, Error> + Send + 'static,
    {
        let observable = self.observable;
        self.scheduler.schedule(async move {
            observable.subscribe(observer).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::observer::*;
    use crate::prelude::*;
    use crate::scheduler;

    #[async_std::test]
    async fn subscribe_on() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_observer = TestObserver::default();
        vec![0, 1, 2, 3]
            .into_observable()
            .subscribe_on(scheduler.clone())
            .subscribe(test_observer.clone()).await;
        scheduler.join();
        assert_eq!(test_observer.status().await, ObserverStatus::Completed);
        assert_eq!(test_observer.items().await, vec![0, 1, 2, 3]);
    }

    #[async_std::test]
    async fn subscribe_on_shared() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_observer = TestObserver::default();
        vec![0, 1, 2, 3]
            .into_observable()
            .subscribe_on(scheduler.clone())
            .subscribe(test_observer.clone()).await;
        scheduler.join();
        assert_eq!(test_observer.status().await, ObserverStatus::Completed);
        assert_eq!(test_observer.items().await, vec![0, 1, 2, 3]);
    }
}
