use crate::core;
use crate::flow;
use std::cell::RefCell;
use std::rc::Rc;

pub struct TestSubscriber<'o, Subscription, Item, Error> {
    subscription: Rc<RefCell<Option<Subscription>>>,
    data: Rc<RefCell<Data<'o, Item, Error>>>,
}

struct Data<'o, Item, Error> {
    items: Vec<Item>,
    error: Option<flow::Error<Error>>,
    is_completed: bool,
    request_on_subscribe: usize,
    request_on_next: usize,
    execute_on_next: Option<Box<dyn FnOnce() + 'o>>,
}

impl<'o, Subscription, Item, Error> Default for TestSubscriber<'o, Subscription, Item, Error> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<'o, Subscription, Item, Error> TestSubscriber<'o, Subscription, Item, Error> {
    pub fn new(request_on_subscribe: usize) -> Self {
        Self {
            subscription: Rc::new(RefCell::new(None)),
            data: Rc::new(RefCell::new(Data {
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

impl<'o, Subscription, Item, Error> Clone for TestSubscriber<'o, Subscription, Item, Error> {
    fn clone(&self) -> Self {
        Self {
            subscription: self.subscription.clone(),
            data: self.data.clone(),
        }
    }
}

pub type SubscriberStatus = crate::util::DownstreamStatus;

impl<'o, Subscription, Item, Error> TestSubscriber<'o, Subscription, Item, Error>
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
        self.subscription.borrow().is_some()
    }

    pub fn cancel(&mut self) {
        assert!(self.is_subscribed());
        self.subscription.borrow().as_ref().unwrap().cancel();
    }

    pub fn request_direct(&self, count: usize) {
        assert_ne!(self.status(), SubscriberStatus::Unsubscribed);
        assert_ne!(self.status(), SubscriberStatus::Cancelled);
        self.subscription.borrow().as_ref().unwrap().request(count)
    }

    pub fn request_on_next(&mut self, count: usize) {
        self.data.borrow_mut().request_on_next += count;
    }

    pub fn execute_on_next<Fn>(&mut self, f: Fn)
    where
        Fn: FnOnce() + 'o,
    {
        self.data.borrow_mut().execute_on_next = Some(Box::new(f));
    }

    pub fn has_error(&self) -> bool {
        self.data.borrow().error.is_some()
    }

    pub fn is_cancelled(&self) -> bool {
        self.subscription
            .borrow()
            .as_ref()
            .map(|subscription| subscription.is_cancelled())
            .unwrap_or(false)
    }

    pub fn is_completed(&self) -> bool {
        self.data.borrow().is_completed
    }
}

impl<'o, Subscription, Item, Error> TestSubscriber<'o, Subscription, Item, Error>
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

impl<'o, Subscription, Item, Error> core::Subscriber<Subscription, Item, Error>
    for TestSubscriber<'o, Subscription, Item, Error>
where
    Subscription: core::Subscription,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        assert_eq!(self.status(), SubscriberStatus::Unsubscribed);
        subscription.request(self.data.borrow().request_on_subscribe);
        *self.subscription.borrow_mut() = Some(subscription);
    }
    fn on_next(&mut self, item: Item) {
        assert_eq!(self.status(), SubscriberStatus::Subscribed);
        let mut data = self.data.borrow_mut();
        data.items.push(item);
        if let Some(f) = data.execute_on_next.take() {
            f()
        };
        self.subscription
            .borrow()
            .as_ref()
            .unwrap()
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
