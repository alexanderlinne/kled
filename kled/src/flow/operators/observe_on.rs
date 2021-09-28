use crate::flow::operators::{Dematerialize, Materialize};
use crate::flow::Signal;
use crate::subscriber::ScheduledSubscriber;
use crate::{core, Never};

#[operator(
    type = "flow",
    subscriber = "ScheduledSubscriber",
    upstream_subscription = "Never",
    upstream_item = "Signal<Subscription, Item, Error>",
    upstream_error = "Never",
    subscription = "Never",
    item = "Signal<Subscription, Item, Error>",
    error = "Never"
)]
pub struct ObserveOnRaw<Scheduler>
where
    Scheduler: core::Scheduler,
{
    scheduler: Scheduler,
}

pub type ObserveOn<Upstream, Subscription, Item, Error, Scheduler> = Dematerialize<
    ObserveOnRaw<
        Materialize<Upstream, Subscription, Item, Error>,
        Subscription,
        Item,
        Error,
        Scheduler,
    >,
    Subscription,
    Item,
    Error,
>;

#[cfg(test)]
mod tests {
    use crate::flow::*;
    use crate::prelude::*;
    use crate::scheduler;
    use crate::subscriber::*;

    #[async_std::test]
    async fn observe_on() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        vec![0, 1, 2, 3]
            .into_flow()
            .observe_on(scheduler.clone())
            .into_step_verifier()
            .expect_subscription()
            .expect_all_of(vec![0, 1, 2, 3])
            .expect_completed()
            .verify()
            .await;
        scheduler.join();
    }

    #[async_std::test]
    async fn observe_on_error() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default().annotate_item_type(());
        test_flow
            .clone()
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone())
            .await;
        test_flow.emit_error(()).await;
        scheduler.join();
        assert_eq!(test_subscriber.status().await, SubscriberStatus::Error);
        assert_eq!(
            test_subscriber.error().await,
            Some(flow::Error::Upstream(()))
        );
    }
}
