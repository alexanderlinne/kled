use async_trait::async_trait;
#[chronobreak]
use std::time::*;
use futures::prelude::*;
use crate::{core, util, flow};
use crate::scheduler::{DelaySender, DelayReceiver, unbounded};
use crate::flow::Signal;

pub struct ScheduledSubscriber<Subscription, Item, Error> {
    sender: Option<DelaySender<Signal<Subscription, Item, Error>>>,
}

impl<Subscription, Item, Error>
    ScheduledSubscriber<Subscription, Item, Error>
{
    pub fn new<Subscriber, Scheduler>(subscriber: Subscriber, scheduler: Scheduler) -> Self
    where
        Subscription: Send + 'static,
        Item: Send + 'static,
        Error: Send + 'static,
        Subscriber: core::Subscriber<util::Never, Signal<Subscription, Item, Error>, util::Never> + Send + 'static,
        Scheduler: core::Scheduler + Send + 'static,
    {
        let (sender, receiver) = unbounded();
        scheduler.schedule(async move {
            let mut subscriber = subscriber;
            let mut receiver: DelayReceiver<Signal<Subscription, Item, Error>> = receiver;
            let mut is_error = false;
            while let Some(signal) = receiver.next().await {
                is_error = signal.is_error();
                subscriber.on_next(signal).await;
            }
            if !is_error {
                subscriber.on_completed().await;
            }
        });
        Self {
            sender: Some(sender),
        }
    }

    pub async fn on_next_delayed(&mut self, delay: Duration, signal: Signal<Subscription, Item, Error>) {
        const MSG: &str = "ScheduledSubscriberRaw::on_next_delayed: upstream called on_next after completion";
        let is_error = signal.is_error();
        self.sender.as_mut().expect(MSG).send_delayed(delay, signal).await.unwrap();
        if is_error {
            self.sender = None;
        }
    }
}

#[async_trait]
impl<Subscription, Item, Error> core::Subscriber<util::Never, Signal<Subscription, Item, Error>, util::Never>
    for ScheduledSubscriber<Subscription, Item, Error>
where
    Subscription: Send,
    Item: Send,
    Error: Send,
{
    async fn on_subscribe(&mut self, _: util::Never) {
        unreachable! {};
    }

    async fn on_next(&mut self, signal: Signal<Subscription, Item, Error>) {
        const MSG: &str = "ScheduledSubscriberRaw::on_next: upstream called on_next after completion";
        let is_error = signal.is_error();
        self.sender.as_mut().expect(MSG).send(signal).await.unwrap();
        if is_error {
            self.sender = None;
        }
    }

    async fn on_error(&mut self, _: flow::Error<util::Never>) {
        unreachable! {};
    }

    async fn on_completed(&mut self) {
        self.sender = None;
    }
}
