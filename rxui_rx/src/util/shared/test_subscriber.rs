use crate::core;
use crate::flow;
use std::sync::{Arc, Mutex};

pub struct TestSubscriber<Subscription, Item, Error> {
    data: Arc<Mutex<Data<Subscription, Item, Error>>>,
}

struct Data<Subscription, Item, Error> {
    subscription: Option<Subscription>,
    items: Vec<Item>,
    error: Option<flow::Error<Error>>,
    is_completed: bool,
    is_cancelled: bool,
    request_on_subscribe: usize,
    request_on_next: usize,
}

impl<Subscription, Item, Error> Default for TestSubscriber<Subscription, Item, Error> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<Subscription, Item, Error> TestSubscriber<Subscription, Item, Error> {
    pub fn new(request_on_subscribe: usize) -> Self {
        Self {
            data: Arc::new(Mutex::new(Data {
                subscription: None,
                items: vec![],
                error: None,
                is_completed: false,
                is_cancelled: false,
                request_on_subscribe,
                request_on_next: 0,
            })),
        }
    }
}

impl<Subscription, Item, Error> Clone for TestSubscriber<Subscription, Item, Error> {
    fn clone(&self) -> Self {
        Self {
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
        if self.data.lock().unwrap().is_cancelled {
            return SubscriberStatus::Cancelled;
        }

        if !self.is_subscribed() {
            return SubscriberStatus::Unsubscribed;
        }

        if self.has_error() {
            return SubscriberStatus::Error;
        }

        if self.is_completed() {
            return SubscriberStatus::Completed;
        }

        return SubscriberStatus::Subscribed;
    }

    pub fn is_subscribed(&self) -> bool {
        self.data.lock().unwrap().subscription.is_some()
    }

    pub fn cancel(&mut self) {
        assert!(self.is_subscribed());
        let mut data = self.data.lock().unwrap();
        data.subscription.take().unwrap().cancel();
        data.is_cancelled = true;
    }

    pub fn request_direct(&self, count: usize) {
        assert_eq!(self.status(), SubscriberStatus::Subscribed);
        self.data
            .lock()
            .unwrap()
            .subscription
            .as_ref()
            .expect("")
            .request(count)
    }

    pub fn request_on_next(&mut self, count: usize) {
        self.data.lock().unwrap().request_on_next += count;
    }

    pub fn has_error(&self) -> bool {
        self.data.lock().unwrap().error.is_some()
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
        let mut data = self.data.lock().unwrap();
        subscription.request(data.request_on_subscribe);
        data.subscription = Some(subscription);
    }
    fn on_next(&mut self, item: Item) {
        assert_eq!(self.status(), SubscriberStatus::Subscribed);
        let mut data = self.data.lock().unwrap();
        data.items.push(item);
        data.subscription
            .as_ref()
            .expect("")
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
