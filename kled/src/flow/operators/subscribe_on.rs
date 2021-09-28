use crate::core;
use async_trait::async_trait;
use std::marker::PhantomData;

#[derive(new)]
pub struct SubscribeOn<Flow, Subscription, Item, Error, Scheduler> {
    flow: Flow,
    scheduler: Scheduler,
    phantom: PhantomData<(Subscription, Item, Error)>,
}

#[async_trait]
impl<Flow, Subscription, Item, Error, Scheduler> core::Flow<Subscription, Item, Error>
    for SubscribeOn<Flow, Subscription, Item, Error, Scheduler>
where
    Flow: core::Flow<Subscription, Item, Error> + Send + 'static,
    Subscription: core::Subscription + Send + Sync + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    Scheduler: core::Scheduler + Send + 'static,
{
    async fn subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Subscription, Item, Error> + Send + 'static,
    {
        let flow = self.flow;
        self.scheduler.schedule(flow.subscribe(subscriber));
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::scheduler;

    #[async_std::test]
    async fn subscribe_on() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        vec![0, 1, 2, 3]
            .into_flow()
            .subscribe_on(scheduler.clone())
            .into_step_verifier()
            .expect_subscription()
            .expect_all_of(vec![0, 1, 2, 3])
            .expect_completed()
            .verify()
            .await;
        scheduler.join();
    }
}
