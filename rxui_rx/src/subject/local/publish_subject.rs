use crate::consumer;
use crate::core;
use crate::core::Consumer;
use std::cell::RefCell;
use std::rc::Rc;

struct Data<'o, Subscription, Item, Error> {
    subscription: Option<Subscription>,
    observers: Vec<Box<dyn core::UnsubscribableConsumer<Item, Error> + 'o>>,
}

#[derive(Clone)]
pub struct PublishSubject<'o, Subscription, Item, Error> {
    data: Rc<RefCell<Data<'o, Subscription, Item, Error>>>,
}

impl<'o, Subscription, Item, Error> Default for PublishSubject<'o, Subscription, Item, Error> {
    fn default() -> Self {
        Self {
            data: Rc::new(RefCell::new(Data {
                subscription: None,
                observers: vec![],
            })),
        }
    }
}

impl<'o, Subscription, Item, Error> core::Observer<Subscription, Item, Error>
    for PublishSubject<'o, Subscription, Item, Error>
where
    Item: Copy,
    Error: Copy,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        self.data.borrow_mut().subscription = Some(subscription);
    }

    fn on_next(&mut self, item: Item) {
        &mut self.data.borrow_mut().observers.on_next(item);
    }

    fn on_error(&mut self, error: Error) {
        &mut self.data.borrow_mut().observers.on_error(error);
    }

    fn on_completed(&mut self) {
        &mut self.data.borrow_mut().observers.on_completed();
    }
}

impl<'o, Subscription, Item, Error> core::LocalSubject<'o, Subscription, Item, Error>
    for PublishSubject<'o, Subscription, Item, Error>
where
    Item: Copy + 'o,
    Error: Copy + 'o,
{
}

impl<'o, Subscription, Item, Error> core::LocalObservable<'o>
    for PublishSubject<'o, Subscription, Item, Error>
where
    Item: 'o,
    Error: 'o,
{
    type Subscription = core::LocalSubscription;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Subscription, Self::Item, Self::Error> + 'o,
    {
        self.data
            .borrow_mut()
            .observers
            .push(Box::new(consumer::local::AutoOnSubscribe::new(observer)))
    }
}

impl<'o, Subscription, Item, Error> core::Observable
    for PublishSubject<'o, Subscription, Item, Error>
{
    type Item = Item;
    type Error = Error;
}
