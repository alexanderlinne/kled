use crate::core;
use crate::subscriber::ScheduledSubscriber;

#[operator(type = "flow", subscriber = "ScheduledSubscriber")]
pub struct ObserveOn<Scheduler>
where
    Scheduler: core::Scheduler
{
    scheduler: Scheduler,
}

#[cfg(test)]
mod tests {
    use crate::flow::*;
    use crate::prelude::*;
    use crate::scheduler;
    use crate::subscriber::*;

    #[async_std::test]
    async fn observe_on() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_subscriber = TestSubscriber::new(4);
        vec![0, 1, 2, 3]
            .into_flow()
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone()).await;
        scheduler.join();
        assert_eq!(test_subscriber.status().await, SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items().await, vec![0, 1, 2, 3]);
    }

    #[async_std::test]
    async fn observe_on_shared() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_subscriber = TestSubscriber::new(4);
        vec![0, 1, 2, 3]
            .into_flow()
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone()).await;
        scheduler.join();
        assert_eq!(test_subscriber.status().await, SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items().await, vec![0, 1, 2, 3]);
    }

    #[async_std::test]
    async fn observe_on_error() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default().annotate_item_type(());
        test_flow
            .clone()
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone()).await;
        test_flow.emit_error(()).await;
        scheduler.join();
        assert_eq!(test_subscriber.status().await, SubscriberStatus::Error);
        assert_eq!(test_subscriber.error().await, Some(flow::Error::Upstream(())));
    }
}
