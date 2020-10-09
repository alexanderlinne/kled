use crate::core;

#[derive(new)]
pub struct ObservableSubscribeOn<Observable, Scheduler>
where
    Observable: core::Observable,
    Scheduler: core::Scheduler + Send + 'static,
{
    observable: Observable,
    scheduler: Scheduler,
}

impl<Observable, Scheduler> core::Observable for ObservableSubscribeOn<Observable, Scheduler>
where
    Observable: core::Observable + Send + 'static,
    Scheduler: core::Scheduler + Send + 'static,
{
    type Item = Observable::Item;
    type Error = Observable::Error;
    type Cancellable = Observable::Cancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + Send + 'static,
    {
        let observable = self.observable;
        self.scheduler.schedule_fn(move || {
            observable.actual_subscribe(observer);
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
