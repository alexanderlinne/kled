use crate::core;
use crate::flow;
use crate::subscription::*;
use std::marker::PhantomData;

pub struct Emitter<Subscriber, Item, Error> {
    subscriber: Subscriber,
    stub: ArcSubscriptionStub,
    phantom: PhantomData<(Item, Error)>,
}

impl<Subscriber, Item, Error> Emitter<Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<ArcSubscription, Item, Error>,
{
    pub async fn from(mut subscriber: Subscriber) -> Self {
        let stub = ArcSubscriptionStub::default();
        subscriber.on_subscribe(stub.subscription()).await;
        Self {
            subscriber,
            stub,
            phantom: PhantomData,
        }
    }

    pub async fn on_next(&mut self, item: Item) {
        self.subscriber.on_next(item).await;
    }

    pub async fn on_error(&mut self, error: Error) {
        self.subscriber.on_error(flow::Error::Upstream(error)).await;
    }

    pub async fn on_completed(&mut self) {
        self.subscriber.on_completed().await;
    }

    pub fn is_cancelled(&self) -> bool {
        self.stub.is_cancelled()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::subscriber::*;

    #[async_std::test]
    async fn basic() {
        let test_subscriber = TestSubscriber::new(1);
        let scheduler = scheduler::NewThreadScheduler::default();
        vec![0, 1, 2]
            .into_flow()
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone()).await;
        scheduler.join();
        assert_eq!(test_subscriber.status().await, SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items().await, vec![0, 1, 2]);
    }
}
