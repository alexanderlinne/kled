use crate::core;
use crate::flow;
#[chronobreak]
use crossbeam::channel::{unbounded, Receiver, Sender};
#[chronobreak]
use parking_lot::Mutex;
use std::cell::UnsafeCell;
use std::marker::PhantomData;
#[chronobreak]
use std::sync::Arc;

#[derive(new)]
pub struct ObserveOn<Flow, Subscription, Item, Error, Scheduler> {
    flow: Flow,
    scheduler: Scheduler,
    phantom: PhantomData<(Subscription, Item, Error)>,
}

impl<Flow, Subscription, Item, Error, Scheduler> core::Flow<Subscription, Item, Error>
    for ObserveOn<Flow, Subscription, Item, Error, Scheduler>
where
    Flow: core::Flow<Subscription, Item, Error>,
    Subscription: core::Subscription + Send + Sync + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    Scheduler: core::Scheduler + Send + 'static,
{
    fn subscribe<Downstream>(self, downstream: Downstream)
    where
        Downstream: core::Subscriber<Subscription, Item, Error> + Send + 'static,
    {
        self.flow
            .subscribe(ObserveOnSubscriber::new(downstream, self.scheduler));
    }
}

struct ObserveOnSubscriber<Subscriber, Scheduler, Subscription, Item, Error> {
    task: Arc<ObserveOnTaskWrapper<Subscriber, Subscription, Item, Error>>,
    sender: Sender<Item>,
    scheduler: Scheduler,
    phantom: PhantomData<Subscription>,
}

impl<Subscriber, Scheduler, Subscription, Item, Error>
    ObserveOnSubscriber<Subscriber, Scheduler, Subscription, Item, Error>
where
    Subscription: Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    Subscriber: core::Subscriber<Subscription, Item, Error> + Send + 'static,
    Scheduler: core::Scheduler + Send + 'static,
{
    fn new(subscriber: Subscriber, scheduler: Scheduler) -> Self {
        let (sender, receiver) = unbounded();
        ObserveOnSubscriber {
            task: Arc::new(ObserveOnTaskWrapper::new(receiver, subscriber)),
            sender,
            scheduler,
            phantom: PhantomData,
        }
    }

    fn schedule<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Data<Error>),
    {
        let last_pending_count = unsafe {
            let mut data = (*self.task.inner.get()).data.lock();
            let last_pending_count = data.pending_count;
            f(&mut data);
            last_pending_count
        };
        if last_pending_count == 0 {
            let task = self.task.clone();
            self.scheduler.schedule_fn(move || unsafe {
                (*task.inner.get()).drain();
            });
        }
    }
}

impl<Subscription, Item, Error, Subscriber, Scheduler> core::Subscriber<Subscription, Item, Error>
    for ObserveOnSubscriber<Subscriber, Scheduler, Subscription, Item, Error>
where
    Subscription: Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    Subscriber: core::Subscriber<Subscription, Item, Error> + Send + 'static,
    Scheduler: core::Scheduler + Send + 'static,
{
    fn on_subscribe(&mut self, cancellable: Subscription) {
        unsafe {
            (*self.task.inner.get())
                .subscriber
                .on_subscribe(cancellable);
        }
    }

    fn on_next(&mut self, item: Item) {
        self.sender.send(item).unwrap();
        self.schedule(|data| data.pending_count += 1);
    }

    fn on_error(&mut self, error: flow::Error<Error>) {
        self.schedule(move |data| {
            data.error = Some(error);
            data.done = true;
        });
    }

    fn on_completed(&mut self) {
        self.schedule(|data| data.done = true);
    }
}

struct ObserveOnTaskWrapper<Subscriber, Subscription, Item, Error> {
    inner: UnsafeCell<ObserveOnTask<Subscriber, Subscription, Item, Error>>,
}

impl<Subscriber, Subscription, Item, Error>
    ObserveOnTaskWrapper<Subscriber, Subscription, Item, Error>
{
    pub fn new(receiver: Receiver<Item>, subscriber: Subscriber) -> Self {
        Self {
            inner: UnsafeCell::new(ObserveOnTask::new(receiver, subscriber)),
        }
    }
}

unsafe impl<Subscriber, Subscription, Item, Error> Send
    for ObserveOnTaskWrapper<Subscriber, Subscription, Item, Error>
{
}
unsafe impl<Subscriber, Subscription, Item, Error> Sync
    for ObserveOnTaskWrapper<Subscriber, Subscription, Item, Error>
{
}

struct ObserveOnTask<Subscriber, Subscription, Item, Error> {
    receiver: Receiver<Item>,
    data: Mutex<Data<Error>>,
    subscriber: Subscriber,
    phantom: PhantomData<Subscription>,
}

impl<Subscriber, Subscription, Item, Error> ObserveOnTask<Subscriber, Subscription, Item, Error> {
    pub fn new(receiver: Receiver<Item>, subscriber: Subscriber) -> Self {
        Self {
            receiver,
            data: Mutex::new(Data::default()),
            subscriber,
            phantom: PhantomData,
        }
    }
}

struct Data<Error> {
    pending_count: usize,
    done: bool,
    error: Option<flow::Error<Error>>,
}

impl<Error> Default for Data<Error> {
    fn default() -> Self {
        Self {
            pending_count: 0,
            done: false,
            error: None,
        }
    }
}

impl<Subscriber, Subscription, Item, Error> ObserveOnTask<Subscriber, Subscription, Item, Error>
where
    Subscriber: core::Subscriber<Subscription, Item, Error>,
{
    pub fn drain(&mut self) {
        let mut expected_count: usize = 0;
        loop {
            for _ in 0..expected_count {
                let mut data = self.data.lock();
                if data.done && data.error.is_some() {
                    self.subscriber.on_error(data.error.take().unwrap());
                    return;
                }
                drop(data);

                let item = self.receiver.recv().unwrap();
                self.subscriber.on_next(item);
            }

            let mut data = self.data.lock();
            data.pending_count -= expected_count;
            expected_count = data.pending_count;
            if expected_count == 0 {
                if data.done {
                    if data.error.is_some() {
                        self.subscriber.on_error(data.error.take().unwrap());
                    } else {
                        self.subscriber.on_completed();
                    }
                }
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::flow::*;
    use crate::prelude::*;
    use crate::scheduler;
    use crate::subscriber::*;

    #[test]
    fn observe_on() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_subscriber = TestSubscriber::new(4);
        vec![0, 1, 2, 3]
            .into_flow()
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone());
        scheduler.join();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn observe_on_shared() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_subscriber = TestSubscriber::new(4);
        vec![0, 1, 2, 3]
            .into_flow()
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone());
        scheduler.join();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn observe_on_error() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default().annotate_item_type(());
        test_flow
            .clone()
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone());
        test_flow.emit_error(());
        scheduler.join();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.error(), Some(flow::Error::Upstream(())));
    }
}
