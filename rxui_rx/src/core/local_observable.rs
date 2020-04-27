use crate::core;
use std::cell::RefCell;
use std::rc::Rc;

pub trait LocalObservable<'o>: core::Observable {
    type Subscription;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Subscription, Self::Item, Self::Error> + 'o;
}

pub struct LocalSubscription {
    subscribed: Rc<RefCell<bool>>,
}

impl LocalSubscription {
    pub(crate) fn new(subscribed: Rc<RefCell<bool>>) -> Self {
        Self { subscribed }
    }
}

impl core::Subscription for LocalSubscription {
    fn unsubscribe(self) {
        *self.subscribed.borrow_mut() = false;
    }
}
