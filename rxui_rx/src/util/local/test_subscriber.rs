use crate::core;
use crate::flow;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct TestSubscriber<Subscription, Item, Error> {
    data: Rc<RefCell<Data<Subscription, Item, Error>>>,
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
    fn new(request_on_subscribe: usize) -> Self {
        Self {
            data: Rc::new(RefCell::new(Data {
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

pub type SubscriberStatus = crate::util::DownstreamStatus;

impl<Subscription, Item, Error> TestSubscriber<Subscription, Item, Error>
where
    Subscription: core::Subscription,
{
    pub fn status(&self) -> SubscriberStatus {
        if self.data.borrow().is_cancelled {
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
        self.data.borrow().subscription.is_some()
    }

    pub fn cancel(&mut self) {
        assert!(self.is_subscribed());
        let mut data = self.data.borrow_mut();
        data.subscription.take().unwrap().cancel();
        data.is_cancelled = true;
    }

    pub fn request_on_next(&mut self, count: usize) {
        self.data.borrow_mut().request_on_next += count;
    }

    pub fn has_error(&self) -> bool {
        self.data.borrow().error.is_some()
    }

    pub fn is_completed(&self) -> bool {
        self.data.borrow().is_completed
    }
}

impl<Subscription, Item, Error> TestSubscriber<Subscription, Item, Error>
where
    Item: Clone,
    Error: Clone,
{
    pub fn items(&self) -> Vec<Item> {
        self.data.borrow().items.clone()
    }

    pub fn error(&self) -> Option<flow::Error<Error>> {
        self.data.borrow().error.clone()
    }
}

impl<Subscription, Item, Error> core::Subscriber<Subscription, Item, Error>
    for TestSubscriber<Subscription, Item, Error>
where
    Subscription: core::Subscription,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        assert_eq!(self.status(), SubscriberStatus::Unsubscribed);
        let mut data = self.data.borrow_mut();
        subscription.request(data.request_on_subscribe);
        data.subscription = Some(subscription);
    }
    fn on_next(&mut self, item: Item) {
        assert_eq!(self.status(), SubscriberStatus::Subscribed);
        let mut data = self.data.borrow_mut();
        data.items.push(item);
        data.subscription
            .as_ref()
            .expect("")
            .request(data.request_on_next);
        data.request_on_next = 0;
    }
    fn on_error(&mut self, error: flow::Error<Error>) {
        assert_eq!(self.status(), SubscriberStatus::Subscribed);
        self.data.borrow_mut().error = Some(error)
    }
    fn on_completed(&mut self) {
        assert_eq!(self.status(), SubscriberStatus::Subscribed);
        self.data.borrow_mut().is_completed = true;
    }
}
