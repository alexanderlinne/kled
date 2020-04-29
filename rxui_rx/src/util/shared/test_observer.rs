use crate::core;
use std::sync::{Arc, Mutex};

pub struct TestObserver<Subscription, Item, Error> {
    data: Arc<Mutex<Data<Subscription, Item, Error>>>,
}

struct Data<Subscription, Item, Error> {
    subscription: Option<Subscription>,
    items: Vec<Item>,
    error: Option<Error>,
    is_completed: bool,
    is_cancelled: bool,
}

impl<Subscription, Item, Error> Default for TestObserver<Subscription, Item, Error> {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(Data {
                subscription: None,
                items: vec![],
                error: None,
                is_completed: false,
                is_cancelled: false,
            })),
        }
    }
}

impl<Subscription, Item, Error> Clone for TestObserver<Subscription, Item, Error> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ObserverStatus {
    Unsubscribed,
    Subscribed,
    Error,
    Completed,
    Cancelled,
}

impl<Subscription, Item, Error> TestObserver<Subscription, Item, Error>
where
    Subscription: core::ObservableSubscription,
{
    pub fn status(&self) -> ObserverStatus {
        if self.data.lock().unwrap().is_cancelled {
            return ObserverStatus::Cancelled;
        }

        if !self.is_subscribed() {
            return ObserverStatus::Unsubscribed;
        }

        if self.has_error() {
            return ObserverStatus::Error;
        }

        if self.is_completed() {
            return ObserverStatus::Completed;
        }

        return ObserverStatus::Subscribed;
    }

    pub fn is_subscribed(&self) -> bool {
        self.data.lock().unwrap().subscription.is_some()
    }

    pub fn cancel_subscription(&mut self) {
        assert!(self.is_subscribed());
        let mut data = self.data.lock().unwrap();
        data.subscription.take().unwrap().cancel();
        data.is_cancelled = true;
    }

    pub fn has_error(&self) -> bool {
        self.data.lock().unwrap().error.is_some()
    }

    pub fn is_completed(&self) -> bool {
        self.data.lock().unwrap().is_completed
    }
}

impl<Subscription, Item, Error> TestObserver<Subscription, Item, Error>
where
    Item: Clone,
    Error: Clone,
{
    pub fn items(&self) -> Vec<Item> {
        self.data.lock().unwrap().items.clone()
    }

    pub fn error(&self) -> Option<Error> {
        self.data.lock().unwrap().error.clone()
    }
}

impl<Subscription, Item, Error> core::Observer<Subscription, Item, Error>
    for TestObserver<Subscription, Item, Error>
where
    Subscription: core::ObservableSubscription,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        assert_eq!(self.status(), ObserverStatus::Unsubscribed);
        self.data.lock().unwrap().subscription = Some(subscription);
    }
    fn on_next(&mut self, item: Item) {
        assert_eq!(self.status(), ObserverStatus::Subscribed);
        self.data.lock().unwrap().items.push(item);
    }
    fn on_error(&mut self, error: Error) {
        assert_eq!(self.status(), ObserverStatus::Subscribed);
        self.data.lock().unwrap().error = Some(error)
    }
    fn on_completed(&mut self) {
        assert_eq!(self.status(), ObserverStatus::Subscribed);
        self.data.lock().unwrap().is_completed = true;
    }
}
