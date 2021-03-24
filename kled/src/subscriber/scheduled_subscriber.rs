use async_trait::async_trait;
#[chronobreak]
use std::time::*;
use futures::prelude::*;
use std::marker::PhantomData;
use crate::{core, scheduler::{DelaySender, unbounded}, signal::Signal, flow};

pub struct ScheduledSubscriber<Subscription, Item, Error> {
    sender: DelaySender<Signal<Subscription, Item, flow::Error<Error>>>,
    phantom: PhantomData<(Subscription, Item, Error)>,
}

impl<Subscription, Item, Error>
    ScheduledSubscriber<Subscription, Item, Error>
{
    pub fn new<Subscriber, Scheduler>(subscriber: Subscriber, scheduler: Scheduler) -> Self
    where
        Subscription: Send + 'static,
        Item: Send + 'static,
        Error: Send + 'static,
        Subscriber: core::Subscriber<Subscription, Item, Error> + Send + 'static,
        Scheduler: core::Scheduler + Send + 'static,
    {
        let (sender, mut receiver) = unbounded();
        scheduler.schedule(async move {
            let mut subscriber = Some(subscriber);
            while let Some(signal) = receiver.next().await {
                match signal {
                    Signal::Subscribe(subscription) => {
                        const MSG: &str = "Subscriber::observe_on: upstream completed before on_subscribe";
                        subscriber.as_mut().expect(MSG).on_subscribe(subscription).await;
                    }
                    Signal::Item(item) => {
                        if let Some(subscriber) = subscriber.as_mut() {
                            subscriber.on_next(item).await;
                        }
                    },
                    Signal::Error(err) => {
                        const MSG: &str = "Subscriber::observe_on: upstream called on_error after completion";
                        subscriber.as_mut().expect(MSG).on_error(err).await;
                        subscriber = None;
                    },
                    Signal::Completed => {
                        const MSG: &str = "Subscriber::observe_on: upstream called on_completed after completion";
                        subscriber.as_mut().expect(MSG).on_completed().await;
                        subscriber = None;
                    }
                }
            }
        });
        Self {
            sender,
            phantom: PhantomData,
        }
    }

    pub async fn on_next_delayed(&mut self, delay: Duration, item: Item) {
        self.sender.send_delayed(delay, Signal::Item(item)).await.unwrap();
    }

    pub async fn on_error_delayed(&mut self, delay: Duration, error: flow::Error<Error>) {
        self.sender.send_delayed(delay, Signal::Error(error)).await.unwrap();
    }

    pub async fn on_completed_delayed(&mut self, delay: Duration) {
        self.sender.send_delayed(delay, Signal::Completed).await.unwrap();
    }
}

#[async_trait]
impl<Subscription, Item, Error> core::Subscriber<Subscription, Item, Error>
    for ScheduledSubscriber<Subscription, Item, Error>
where
    Subscription: Send,
    Item: Send,
    Error: Send,
{
    async fn on_subscribe(&mut self, subscription: Subscription) {
        self.sender.send_direct(Signal::Subscribe(subscription)).await.unwrap();
    }

    async fn on_next(&mut self, item: Item) {
        self.sender.send(Signal::Item(item)).await.unwrap();
    }

    async fn on_error(&mut self, error: flow::Error<Error>) {
        self.sender.send_direct(Signal::Error(error)).await.unwrap();
    }

    async fn on_completed(&mut self) {
        self.sender.send_direct(Signal::Completed).await.unwrap();
    }
}
