#[chronobreak]
use std::time::*;
use std::pin::Pin;
use futures::prelude::*;
use futures::task::{Context, Poll};
use futures::executor;
use std::marker::PhantomData;
use pin_project::pin_project;
use crate::{core, scheduler::{DelaySender, DelayReceiver, unbounded}, signal::Signal};

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
        let (sender, receiver) = unbounded();
        scheduler.schedule(Task {
            receiver,
            observer: Some(observer),
        });
        Self {
            sender,
            phantom: PhantomData,
        }
    }

    pub fn on_next_delayed(&mut self, delay: Duration, item: Item) {
        executor::block_on(self.sender.send_delayed(delay, Signal::Item(item))).unwrap();
    }

    pub fn on_error_delayed(&mut self, delay: Duration, error: Error) {
        executor::block_on(self.sender.send_delayed(delay, Signal::Err(error))).unwrap();
    }

    pub fn on_completed_delayed(&mut self, delay: Duration) {
        executor::block_on(self.sender.send_delayed(delay, Signal::Completed)).unwrap();
    }
}

impl<Cancellable, Item, Error> core::Observer<Cancellable, Item, Error>
    for ScheduledObserver<Cancellable, Item, Error>
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        executor::block_on(self.sender.send_direct(Signal::Subscribe(cancellable))).unwrap();
    }

    fn on_next(&mut self, item: Item) {
        executor::block_on(self.sender.send(Signal::Item(item))).unwrap();
    }

    fn on_error(&mut self, error: Error) {
        executor::block_on(self.sender.send_direct(Signal::Err(error))).unwrap();
    }

    fn on_completed(&mut self) {
        executor::block_on(self.sender.send_direct(Signal::Completed)).unwrap();
    }
}

#[pin_project]
struct Task<Observer, Cancellable, Item, Error> {
    #[pin]
    receiver: DelayReceiver<Signal<Cancellable, Item, Error>>,
    observer: Option<Observer>,
}

impl<Observer, Cancellable, Item, Error> Future for Task<Observer, Cancellable, Item, Error>
where
    Cancellable: Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    Observer: core::Observer<Cancellable, Item, Error> + Send + 'static,
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
                Signal::Subscribe(cancellable) => {
                    const MSG: &str = "Observer::observe_on: upstream completed before on_subscribe";
                    this.observer.as_mut().expect(MSG).on_subscribe(cancellable);
                }
                Signal::Item(item) => {
                    if let Some(observer) = this.observer.as_mut() {
                        observer.on_next(item);
                    }
                },
                Signal::Err(err) => {
                    const MSG: &str = "Observer::observe_on: upstream called on_error after completion";
                    this.observer.as_mut().expect(MSG).on_error(err);
                    *this.observer = None;
                },
                Signal::Completed => {
                    const MSG: &str = "Observer::observe_on: upstream called on_completed after completion";
                    this.observer.as_mut().expect(MSG).on_completed();
                    *this.observer = None;
                }
            }
        }
        Poll::Ready(())
    }
}
