#[chronobreak]
use std::time::*;
use crate::scheduler::Worker;
use std::marker::PhantomData;
use crate::core;
use crate::flow;

pub struct ScheduledSubscriber<Subscriber, Subscription, Item, Error> {
    subscriber: Option<Subscriber>,
    worker: Option<Worker<Option<Subscriber>>>,
    phantom: PhantomData<(Subscription, Item, Error)>,
}

impl<Subscriber, Subscription, Item, Error>
    ScheduledSubscriber<Subscriber, Subscription, Item, Error>
where
    Subscription: Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    Subscriber: core::Subscriber<Subscription, Item, Error> + Send + 'static,
{
    pub fn new<Scheduler>(subscriber: Subscriber, scheduler: Scheduler) -> Self
    where
        Scheduler: core::Scheduler + Send + 'static,
    {
        Self {
            subscriber: Some(subscriber),
            worker: Some(scheduler.worker(None)),
            phantom: PhantomData,
        }
    }

    pub fn on_next_delayed(&mut self, delay: Duration, item: Item) {
        if let Some(worker) = self.worker.as_mut() {
            worker.schedule_delayed(delay, move |state| {
                if let Some(subscriber) = state {
                    subscriber.on_next(item);
                }
            })
        };
    }

    pub fn on_error_delayed(&mut self, delay: Duration, error: flow::Error<Error>) {
        const MSG: &str = "Flow::observe_on: upstream called on_error after completion";
        self.worker.as_mut().expect(MSG).schedule_delayed(delay, |state| {
            state.as_mut().expect(MSG).on_error(error);
        });
        self.worker = None;
    }

    pub fn on_completed_delayed(&mut self, delay: Duration) {
        const MSG: &str = "Flow::observe_on: upstream called on_completed after completion";
        self.worker.as_mut().expect(MSG).schedule_delayed(delay, |state| {
            state.as_mut().expect(MSG).on_completed();
        });
        self.worker = None;
    }
}

impl<Subscription, Item, Error, Subscriber> core::Subscriber<Subscription, Item, Error>
    for ScheduledSubscriber<Subscriber, Subscription, Item, Error>
where
    Subscription: Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    Subscriber: core::Subscriber<Subscription, Item, Error> + Send + 'static,
{
    fn on_subscribe(&mut self, cancellable: Subscription) {
        const MSG: &str = "Flow::observe_on: upstream completed before on_subscribe";
        let mut subscriber = self.subscriber.take();
        subscriber.as_mut().expect(MSG).on_subscribe(cancellable);
        self.worker.as_mut().expect(MSG).schedule_direct(move |state| {
            *state = subscriber;
        })
    }

    fn on_next(&mut self, item: Item) {
        println!("next");
        if let Some(worker) = self.worker.as_mut() { 
            worker.schedule(move |state| {
                if let Some(subscriber) = state {
                    subscriber.on_next(item);
                }
            })
        };
    }

    fn on_error(&mut self, error: flow::Error<Error>) {
        println!("error");
        const MSG: &str = "Flow::observe_on: upstream called on_error after completion";
        self.worker.as_mut().expect(MSG).schedule_direct(|state| {
            state.as_mut().expect(MSG).on_error(error);
        });
        self.worker = None;
    }

    fn on_completed(&mut self) {
        println!("completed");
        const MSG: &str = "Flow::observe_on: upstream called on_completed after completion";
        self.worker.as_mut().expect(MSG).schedule_direct(|state| {
            state.as_mut().expect(MSG).on_completed();
        });
        self.worker = None;
    }
}
