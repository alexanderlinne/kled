use crate::core;
use std::marker::PhantomData;

#[derive(new)]
pub struct ObservableSubscribeOn<Observable, Cancellable, Item, Error, Scheduler>
where
    Observable: core::Observable<Cancellable, Item, Error>,
    Scheduler: core::Scheduler + Send + 'static,
{
    observable: Observable,
    scheduler: Scheduler,
    phantom: PhantomData<(Cancellable, Item, Error)>,
}

impl<Observable, Cancellable, Item, Error, Scheduler> core::Observable<Cancellable, Item, Error>
    for ObservableSubscribeOn<Observable, Cancellable, Item, Error, Scheduler>
where
    Observable: core::Observable<Cancellable, Item, Error> + Send + 'static,
    Cancellable: core::Cancellable,
    Scheduler: core::Scheduler + Send + 'static,
{
    fn subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Cancellable, Item, Error> + Send + 'static,
    {
        let observable = self.observable;
        self.scheduler.schedule_fn(move || {
            observable.subscribe(observer);
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::observer::*;
    use crate::prelude::*;
    use crate::scheduler;

    #[test]
    fn subscribe_on() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_observer = TestObserver::default();
        vec![0, 1, 2, 3]
            .into_observable()
            .subscribe_on(scheduler.clone())
            .subscribe(test_observer.clone());
        scheduler.join();
        assert_eq!(test_observer.status(), ObserverStatus::Completed);
        assert_eq!(test_observer.items(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn subscribe_on_shared() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_observer = TestObserver::default();
        vec![0, 1, 2, 3]
            .into_observable()
            .subscribe_on(scheduler.clone())
            .subscribe(test_observer.clone());
        scheduler.join();
        assert_eq!(test_observer.status(), ObserverStatus::Completed);
        assert_eq!(test_observer.items(), vec![0, 1, 2, 3]);
    }
}
