#[chronobreak]
use std::time::*;
use std::pin::Pin;
use futures::prelude::*;
use futures::task::{Context, Poll};
use futures::executor;
use std::marker::PhantomData;
use pin_project::pin_project;
use crate::{core, scheduler::{DelaySender, DelayReceiver, unbounded}, signal::Signal, flow};

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
        let (sender, receiver) = unbounded();
        scheduler.schedule(Task {
            receiver,
            subscriber: Some(subscriber),
        });
        Self {
            sender,
            phantom: PhantomData,
        }
    }

    pub fn on_next_delayed(&mut self, delay: Duration, item: Item) {
        executor::block_on(self.sender.send_delayed(delay, Signal::Item(item))).unwrap();
    }

    pub fn on_error_delayed(&mut self, delay: Duration, error: flow::Error<Error>) {
        executor::block_on(self.sender.send_delayed(delay, Signal::Err(error))).unwrap();
    }

    pub fn on_completed_delayed(&mut self, delay: Duration) {
        executor::block_on(self.sender.send_delayed(delay, Signal::Completed)).unwrap();
    }
}

impl<Subscription, Item, Error> core::Subscriber<Subscription, Item, Error>
    for ScheduledSubscriber<Subscription, Item, Error>
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        executor::block_on(self.sender.send_direct(Signal::Subscribe(subscription))).unwrap();
    }

    fn on_next(&mut self, item: Item) {
        executor::block_on(self.sender.send(Signal::Item(item))).unwrap();
    }

    fn on_error(&mut self, error: flow::Error<Error>) {
        executor::block_on(self.sender.send_direct(Signal::Err(error))).unwrap();
    }

    fn on_completed(&mut self) {
        executor::block_on(self.sender.send_direct(Signal::Completed)).unwrap();
    }
}

#[pin_project]
struct Task<Subscriber, Subscription, Item, Error> {
    #[pin]
    receiver: DelayReceiver<Signal<Subscription, Item, flow::Error<Error>>>,
    subscriber: Option<Subscriber>,
}

impl<Subscriber, Subscription, Item, Error> Future for Task<Subscriber, Subscription, Item, Error>
where
    Subscription: Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    Subscriber: core::Subscriber<Subscription, Item, Error> + Send + 'static,
{
    type Output = ();
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        loop {
            let recvd = match this.receiver.as_mut().poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(val) => val,
            };
            let signal = match recvd {
                Some(signal) => signal,
                None => break,
            };
            match signal {
                Signal::Subscribe(subscription) => {
                    const MSG: &str = "Subscriber::observe_on: upstream completed before on_subscribe";
                    this.subscriber.as_mut().expect(MSG).on_subscribe(subscription);
                }
                Signal::Item(item) => {
                    if let Some(subscriber) = this.subscriber.as_mut() {
                        subscriber.on_next(item);
                    }
                },
                Signal::Err(err) => {
                    const MSG: &str = "Subscriber::observe_on: upstream called on_error after completion";
                    this.subscriber.as_mut().expect(MSG).on_error(err);
                    *this.subscriber = None;
                },
                Signal::Completed => {
                    const MSG: &str = "Subscriber::observe_on: upstream called on_completed after completion";
                    this.subscriber.as_mut().expect(MSG).on_completed();
                    *this.subscriber = None;
                }
            }
        }
        Poll::Ready(())
    }
}
