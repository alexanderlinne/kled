use async_trait::async_trait;
#[chronobreak]
use std::time::*;
use futures::prelude::*;
use std::marker::PhantomData;
use crate::{core, scheduler::{DelaySender, unbounded}, signal::Signal};

pub struct ScheduledObserver<Cancellable, Item, Error> {
    sender: DelaySender<Signal<Cancellable, Item, Error>>,
    phantom: PhantomData<(Cancellable, Item, Error)>,
}

impl<Cancellable, Item, Error>
    ScheduledObserver<Cancellable, Item, Error>
{
    pub fn new<Observer, Scheduler>(observer: Observer, scheduler: Scheduler) -> Self
    where
        Cancellable: Send + 'static,
        Item: Send + 'static,
        Error: Send + 'static,
        Observer: core::Observer<Cancellable, Item, Error> + Send + 'static,
        Scheduler: core::Scheduler + Send + 'static,
    {
        let (sender, mut receiver) = unbounded();
        scheduler.schedule(async move {
            let mut observer = Some(observer);
            while let Some(signal) = receiver.next().await {
                match signal {
                    Signal::Subscribe(cancellable) => {
                        const MSG: &str = "Observer::observe_on: upstream completed before on_subscribe";
                        observer.as_mut().expect(MSG).on_subscribe(cancellable).await;
                    }
                    Signal::Item(item) => {
                        if let Some(observer) = observer.as_mut() {
                            observer.on_next(item).await;
                        }
                    },
                    Signal::Err(err) => {
                        const MSG: &str = "Observer::observe_on: upstream called on_error after completion";
                        observer.as_mut().expect(MSG).on_error(err).await;
                        observer = None;
                    },
                    Signal::Completed => {
                        const MSG: &str = "Observer::observe_on: upstream called on_completed after completion";
                        observer.as_mut().expect(MSG).on_completed().await;
                        observer = None;
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

    pub async fn on_error_delayed(&mut self, delay: Duration, error: Error) {
        self.sender.send_delayed(delay, Signal::Err(error)).await.unwrap();
    }

    pub async fn on_completed_delayed(&mut self, delay: Duration) {
        self.sender.send_delayed(delay, Signal::Completed).await.unwrap();
    }
}

#[async_trait]
impl<Cancellable, Item, Error> core::Observer<Cancellable, Item, Error>
    for ScheduledObserver<Cancellable, Item, Error>
where
    Cancellable: Send,
    Item: Send,
    Error: Send,
{
    async fn on_subscribe(&mut self, cancellable: Cancellable) {
        self.sender.send_direct(Signal::Subscribe(cancellable)).await.unwrap();
    }

    async fn on_next(&mut self, item: Item) {
        self.sender.send(Signal::Item(item)).await.unwrap();
    }

    async fn on_error(&mut self, error: Error) {
        self.sender.send_direct(Signal::Err(error)).await.unwrap();
    }

    async fn on_completed(&mut self) {
        self.sender.send_direct(Signal::Completed).await.unwrap();
    }
}
