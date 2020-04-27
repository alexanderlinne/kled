use crate::core;
use crate::core::AutoUnsubscribeObserver;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct AutoUnscubscribe<ObserverType, Subscription, Item, Error> {
    observer: ObserverType,
    subscribed: Rc<RefCell<bool>>,
    phantom: PhantomData<(Subscription, Item, Error)>,
}

impl<'o, ObserverType, Subscription, Item, Error>
    AutoUnscubscribe<ObserverType, Subscription, Item, Error>
where
    ObserverType: core::Observer<Subscription, Item, Error> + 'o,
{
    pub fn new(observer: ObserverType) -> Self {
        Self {
            observer,
            subscribed: Rc::new(RefCell::new(true)),
            phantom: PhantomData,
        }
    }

    pub fn create_subscription(&self) -> core::LocalSubscription {
        core::LocalSubscription::new(self.subscribed.clone())
    }
}

impl<'o, ObserverType, Subscription, Item, Error>
    core::AutoUnsubscribeObserver<Subscription, Item, Error>
    for AutoUnscubscribe<ObserverType, Subscription, Item, Error>
where
    ObserverType: core::Observer<Subscription, Item, Error> + 'o,
{
    fn is_unsubscribed(&self) -> bool {
        !*self.subscribed.borrow()
    }
}

impl<'o, ObserverType, Subscription, Item, Error> core::Observer<Subscription, Item, Error>
    for AutoUnscubscribe<ObserverType, Subscription, Item, Error>
where
    ObserverType: core::Observer<Subscription, Item, Error> + 'o,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        if !self.is_unsubscribed() {
            self.observer.on_subscribe(subscription);
        }
    }

    fn on_next(&mut self, item: Item) {
        if !self.is_unsubscribed() {
            self.observer.on_next(item);
        }
    }

    fn on_error(&mut self, error: Error) {
        if !self.is_unsubscribed() {
            self.observer.on_error(error);
        }
    }

    fn on_completed(&mut self) {
        if !self.is_unsubscribed() {
            self.observer.on_completed();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::observer;
    use crate::prelude::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn unsubscribe() {
        let vec = vec![0, 1, 2, 3];
        let sum = Rc::new(RefCell::new(0));
        let sum_move = sum.clone();
        vec.into_observable().subscribe(observer::from_fn(
            |sub: LocalSubscription| {
                sub.unsubscribe();
            },
            move |v| *sum_move.borrow_mut() += v,
            |_| {},
            || {},
        ));
        assert_eq!(*sum.borrow(), 0);
    }
}
