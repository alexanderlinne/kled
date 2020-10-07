use crate::core;

#[derive(new, reactive_operator)]
pub struct ObservableSubscribeOn<Observable, Scheduler>
where
    Observable: core::SharedObservable,
    Scheduler: core::Scheduler + Send + 'static,
{
    #[upstream(derive_impls = "base")]
    observable: Observable,
    scheduler: Scheduler,
}

impl<Observable, Scheduler> core::SharedObservable for ObservableSubscribeOn<Observable, Scheduler>
where
    Observable: core::SharedObservable + Send + 'static,
    Scheduler: core::Scheduler + Send + 'static,
{
    type Cancellable = Observable::Cancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + Send + 'static,
    {
        let observable = self.observable;
        self.scheduler.schedule(move || {
            observable.actual_subscribe(observer);
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::observer::shared::*;
    use crate::prelude::*;
    use crate::scheduler;

    #[test]
    fn subscribe_on() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_observer = TestObserver::default();
        vec![0, 1, 2, 3]
            .into_shared_observable()
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
            .into_shared_observable()
            .subscribe_on(scheduler.clone())
            .subscribe(test_observer.clone());
        scheduler.join();
        assert_eq!(test_observer.status(), ObserverStatus::Completed);
        assert_eq!(test_observer.items(), vec![0, 1, 2, 3]);
    }
}
