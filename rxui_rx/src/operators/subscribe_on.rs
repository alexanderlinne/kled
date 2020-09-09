use crate::core;

pub struct SubscribeOn<Observable, Worker> {
    observable: Observable,
    worker: Worker,
}

impl<Observable, Worker> SubscribeOn<Observable, Worker>
where
    Worker: core::Worker + Send + 'static,
{
    pub fn new(observable: Observable, worker: Worker) -> Self {
        Self { observable, worker }
    }
}

impl<Observable, Worker> core::SharedObservable for SubscribeOn<Observable, Worker>
where
    Observable: core::SharedObservable + Send + 'static,
    Worker: core::Worker + Send + 'static,
{
    type Cancellable = Observable::Cancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + Send + 'static,
    {
        let observable = self.observable;
        self.worker.schedule(move || {
            observable.actual_subscribe(observer);
        });
    }
}

impl<Observable, Worker> core::Observable for SubscribeOn<Observable, Worker>
where
    Observable: core::Observable,
{
    type Item = Observable::Item;
    type Error = Observable::Error;
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::scheduler;
    use crate::util::shared::*;

    #[test]
    fn subscribe_on() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_observer = TestObserver::default();
        vec![0, 1, 2, 3]
            .into_observable()
            .subscribe_on(&scheduler)
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
            .subscribe_on(&scheduler)
            .subscribe(test_observer.clone());
        scheduler.join();
        assert_eq!(test_observer.status(), ObserverStatus::Completed);
        assert_eq!(test_observer.items(), vec![0, 1, 2, 3]);
    }
}
