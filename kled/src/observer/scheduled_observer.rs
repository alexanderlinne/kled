#[chronobreak]
use std::time::*;
use crate::scheduler::Worker;
use std::marker::PhantomData;
use crate::core;

pub struct ScheduledObserver<Observer, Cancellable, Item, Error> {
    observer: Option<Observer>,
    worker: Option<Worker<Option<Observer>>>,
    phantom: PhantomData<(Cancellable, Item, Error)>,
}

impl<Observer, Cancellable, Item, Error>
    ScheduledObserver<Observer, Cancellable, Item, Error>
where
    Cancellable: Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    Observer: core::Observer<Cancellable, Item, Error> + Send + 'static,
{
    pub fn new<Scheduler>(observer: Observer, scheduler: Scheduler) -> Self
    where
        Scheduler: core::Scheduler + Send + 'static,
    {
        Self {
            observer: Some(observer),
            worker: Some(scheduler.worker(None)),
            phantom: PhantomData,
        }
    }

    pub fn on_next_delayed(&mut self, delay: Duration, item: Item) {
        if let Some(worker) = self.worker.as_mut() {
            worker.schedule_delayed(delay, move |state| {
                if let Some(observer) = state {
                    observer.on_next(item);
                }
            })
        };
    }

    pub fn on_error_delayed(&mut self, delay: Duration, error: Error) {
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

impl<Cancellable, Item, Error, Observer> core::Observer<Cancellable, Item, Error>
    for ScheduledObserver<Observer, Cancellable, Item, Error>
where
    Cancellable: Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    Observer: core::Observer<Cancellable, Item, Error> + Send + 'static,
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        const MSG: &str = "Flow::observe_on: upstream completed before on_subscribe";
        let mut observer = self.observer.take();
        observer.as_mut().expect(MSG).on_subscribe(cancellable);
        self.worker.as_mut().expect(MSG).schedule_direct(move |state| {
            *state = observer;
        })
    }

    fn on_next(&mut self, item: Item) {
        println!("next");
        if let Some(worker) = self.worker.as_mut() { 
            worker.schedule(move |state| {
                if let Some(observer) = state {
                    observer.on_next(item);
                }
            })
        };
    }

    fn on_error(&mut self, error: Error) {
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
