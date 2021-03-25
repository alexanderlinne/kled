use async_trait::async_trait;
#[chronobreak]
use std::time::*;
use futures::prelude::*;
use crate::{core, util};
use crate::scheduler::{DelaySender, DelayReceiver, unbounded};
use crate::observable::Signal;

pub struct ScheduledObserver<Cancellable, Item, Error> {
    sender: Option<DelaySender<Signal<Cancellable, Item, Error>>>,
}

impl<Cancellable, Item, Error>
    ScheduledObserver<Cancellable, Item, Error>
{
    pub fn new<Observer, Scheduler>(observer: Observer, scheduler: Scheduler) -> Self
    where
        Cancellable: Send + 'static,
        Item: Send + 'static,
        Error: Send + 'static,
        Observer: core::Observer<util::Never, Signal<Cancellable, Item, Error>, util::Never> + Send + 'static,
        Scheduler: core::Scheduler + Send + 'static,
    {
        let (sender, receiver) = unbounded();
        scheduler.schedule(async move {
            let mut observer = observer;
            let mut receiver: DelayReceiver<Signal<Cancellable, Item, Error>> = receiver;
            let mut is_error = false;
            while let Some(signal) = receiver.next().await {
                is_error = signal.is_error();
                observer.on_next(signal).await;
            }
            if !is_error {
                observer.on_completed().await;
            }
        });
        Self {
            sender: Some(sender),
        }
    }

    pub async fn on_next_delayed(&mut self, delay: Duration, signal: Signal<Cancellable, Item, Error>) {
        const MSG: &str = "ScheduledObserverRaw::on_next_delayed: upstream called on_next after completion";
        let is_error = signal.is_error();
        self.sender.as_mut().expect(MSG).send_delayed(delay, signal).await.unwrap();
        if is_error {
            self.sender = None;
        }
    }
}

#[async_trait]
impl<Cancellable, Item, Error> core::Observer<util::Never, Signal<Cancellable, Item, Error>, util::Never>
    for ScheduledObserver<Cancellable, Item, Error>
where
    Cancellable: Send,
    Item: Send,
    Error: Send,
{
    async fn on_subscribe(&mut self, _: util::Never) {
        unreachable! {};
    }

    async fn on_next(&mut self, signal: Signal<Cancellable, Item, Error>) {
        const MSG: &str = "ScheduledObserverRaw::on_next: upstream called on_next after completion";
        let is_error = signal.is_error();
        self.sender.as_mut().expect(MSG).send(signal).await.unwrap();
        if is_error {
            self.sender = None;
        }
    }

    async fn on_error(&mut self, _: util::Never) {
        unreachable! {};
    }

    async fn on_completed(&mut self) {
        self.sender = None;
    }
}
