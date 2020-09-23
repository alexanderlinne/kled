use crate::core;
use crate::flow;
use parking_lot::ReentrantMutex;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

pub struct TestSubscriber<Subscription, Item, Error> {
    subscription: Arc<ReentrantMutex<RefCell<Option<Subscription>>>>,
    data: Arc<Mutex<Data<Item, Error>>>,
}

struct Data<Item, Error> {
    items: Vec<Item>,
    error: Option<flow::Error<Error>>,
    is_completed: bool,
    request_on_subscribe: usize,
    request_on_next: usize,
    execute_on_next: Option<Box<dyn FnOnce() + Send + 'static>>,
}

impl<Subscription, Item, Error> Default for TestSubscriber<Subscription, Item, Error> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<Subscription, Item, Error> TestSubscriber<Subscription, Item, Error> {
    pub fn new(request_on_subscribe: usize) -> Self {
        Self {
            subscription: Arc::new(ReentrantMutex::new(RefCell::new(None))),
            data: Arc::new(Mutex::new(Data {
                items: vec![],
                error: None,
                is_completed: false,
                request_on_subscribe,
                request_on_next: 0,
                execute_on_next: None,
            })),
        }
    }
}

impl<Subscription, Item, Error> Clone for TestSubscriber<Subscription, Item, Error> {
    fn clone(&self) -> Self {
        Self {
            subscription: self.subscription.clone(),
            data: self.data.clone(),
        }
    }
}

pub type SubscriberStatus = crate::util::DownstreamStatus;

impl<Subscription, Item, Error> TestSubscriber<Subscription, Item, Error>
where
    Subscription: core::Subscription,
{
    pub fn status(&self) -> SubscriberStatus {
        if self.is_cancelled() {
            SubscriberStatus::Cancelled
        } else if !self.is_subscribed() {
            SubscriberStatus::Unsubscribed
        } else if self.has_error() {
            SubscriberStatus::Error
        } else if self.is_completed() {
            SubscriberStatus::Completed
        } else {
            SubscriberStatus::Subscribed
        }
    }

    pub fn is_subscribed(&self) -> bool {
        self.subscription.lock().borrow().is_some()
    }

    pub fn cancel(&mut self) {
        assert!(self.is_subscribed());
        self.subscription.lock().borrow().as_ref().unwrap().cancel();
    }

    pub fn request_direct(&self, count: usize) {
        assert_ne!(self.status(), SubscriberStatus::Unsubscribed);
        assert_ne!(self.status(), SubscriberStatus::Cancelled);
        self.subscription
            .lock()
            .borrow()
            .as_ref()
            .unwrap()
            .request(count)
    }

    pub fn request_on_next(&mut self, count: usize) {
        self.data.lock().unwrap().request_on_next += count;
    }

    pub fn execute_on_next<Fn>(&mut self, f: Fn)
    where
        Fn: FnOnce() + Send + 'static,
    {
        self.data.lock().unwrap().execute_on_next = Some(Box::new(f));
    }

    pub fn has_error(&self) -> bool {
        self.data.lock().unwrap().error.is_some()
    }

    pub fn is_cancelled(&self) -> bool {
        self.subscription
            .lock()
            .borrow()
            .as_ref()
            .map(|subscription| subscription.is_cancelled())
            .unwrap_or(false)
    }

    pub fn is_completed(&self) -> bool {
        self.data.lock().unwrap().is_completed
    }
}

impl<Subscription, Item, Error> TestSubscriber<Subscription, Item, Error>
where
    Item: Clone,
    Error: Clone,
{
    pub fn items(&self) -> Vec<Item> {
        self.data.lock().unwrap().items.clone()
    }

    pub fn error(&self) -> Option<flow::Error<Error>> {
        self.data.lock().unwrap().error.clone()
    }
}

impl<Subscription, Item, Error> core::Subscriber<Subscription, Item, Error>
    for TestSubscriber<Subscription, Item, Error>
where
    Subscription: core::Subscription,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        assert_eq!(self.status(), SubscriberStatus::Unsubscribed);
        subscription.request(self.data.lock().unwrap().request_on_subscribe);
        *self.subscription.lock().borrow_mut() = Some(subscription);
    }
    fn on_next(&mut self, item: Item) {
        assert_eq!(self.status(), SubscriberStatus::Subscribed);
        let mut data = self.data.lock().unwrap();
        data.items.push(item);
        if let Some(f) = data.execute_on_next.take() {
            f()
        };
        self.subscription
            .lock()
            .borrow()
            .as_ref()
            .unwrap()
            .request(data.request_on_next);
        data.request_on_next = 0;
    }
    fn on_error(&mut self, error: flow::Error<Error>) {
        assert_eq!(self.status(), SubscriberStatus::Subscribed);
        self.data.lock().unwrap().error = Some(error)
    }
    fn on_completed(&mut self) {
        assert_eq!(self.status(), SubscriberStatus::Subscribed);
        self.data.lock().unwrap().is_completed = true;
    }
}
