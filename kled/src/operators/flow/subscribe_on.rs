use crate::core;

#[derive(new)]
pub struct FlowSubscribeOn<Flow, Scheduler>
where
    Flow: core::Flow,
    Scheduler: core::Scheduler + Send + 'static,
{
    flow: Flow,
    scheduler: Scheduler,
}

impl<Flow, Scheduler> core::Flow for FlowSubscribeOn<Flow, Scheduler>
where
    Flow: core::Flow + Send + 'static,
    Scheduler: core::Scheduler + Send + 'static,
{
    type Item = Flow::Item;
    type Error = Flow::Error;
    type Subscription = Flow::Subscription;

    fn actual_subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Self::Subscription, Self::Item, Self::Error> + Send + 'static,
    {
        let flow = self.flow;
        self.scheduler.schedule(move || {
            flow.actual_subscribe(subscriber);
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::scheduler;
    use crate::subscriber::*;

    #[test]
    fn subscribe_on() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_subscriber = TestSubscriber::default();
        vec![0, 1, 2, 3]
            .into_flow()
            .subscribe_on(scheduler.clone())
            .subscribe(test_subscriber.clone());
        scheduler.join();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn subscribe_on_shared() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_subscriber = TestSubscriber::default();
        vec![0, 1, 2, 3]
            .into_flow()
            .subscribe_on(scheduler.clone())
            .subscribe(test_subscriber.clone());
        scheduler.join();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0, 1, 2, 3]);
    }
}
