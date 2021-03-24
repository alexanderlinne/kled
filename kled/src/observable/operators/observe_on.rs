use crate::core;
use crate::observer::ScheduledObserver;

#[operator(type = "observable", subscriber = "ScheduledObserver")]
pub struct ObserveOn<Scheduler>
where
    Scheduler: core::Scheduler,
{
    scheduler: Scheduler,
}

#[cfg(test)]
mod tests {
    use crate::observable::*;
    use crate::observer::*;
    use crate::prelude::*;
    use crate::scheduler;

    #[async_std::test]
    async fn observe_on() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_observer = TestObserver::default();
        vec![0, 1, 2, 3]
            .into_observable()
            .observe_on(scheduler.clone())
            .subscribe(test_observer.clone()).await;
        scheduler.join();
        assert_eq!(test_observer.status().await, ObserverStatus::Completed);
        assert_eq!(test_observer.items().await, vec![0, 1, 2, 3]);
    }

    #[async_std::test]
    async fn observe_on_shared() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_observer = TestObserver::default();
        vec![0, 1, 2, 3]
            .into_observable()
            .observe_on(scheduler.clone())
            .subscribe(test_observer.clone()).await;
        scheduler.join();
        assert_eq!(test_observer.status().await, ObserverStatus::Completed);
        assert_eq!(test_observer.items().await, vec![0, 1, 2, 3]);
    }

    #[async_std::test]
    async fn observe_on_error() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_observer = TestObserver::default();
        let test_observable = TestObservable::default().annotate_item_type(());
        test_observable
            .clone()
            .observe_on(scheduler.clone())
            .subscribe(test_observer.clone()).await;
        test_observable.emit_error(()).await;
        scheduler.join();
        assert_eq!(test_observer.status().await, ObserverStatus::Error);
        assert_eq!(test_observer.error().await, Some(()));
    }
}
