use crate::consumer;
use crate::core;
use crate::core::Consumer;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct PublishSubject<Subscription, Item, Error> {
    data: Arc<Mutex<Data<Subscription, Item, Error>>>,
}

struct Data<Subscription, Item, Error> {
    subscription: Option<Subscription>,
    observers: Vec<Box<dyn core::CancellableConsumer<Item, Error> + Send + Sync + 'static>>,
}

impl<Subscription, Item, Error> Default for PublishSubject<Subscription, Item, Error> {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(Data {
                subscription: None,
                observers: vec![],
            })),
        }
    }
}

impl<Subscription, Item, Error> core::Observer<Subscription, Item, Error>
    for PublishSubject<Subscription, Item, Error>
where
    Item: Copy,
    Error: Copy,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        self.data.lock().unwrap().subscription = Some(subscription);
    }

    fn on_next(&mut self, item: Item) {
        &mut self.data.lock().unwrap().observers.on_next(item);
    }

    fn on_error(&mut self, error: Error) {
        &mut self.data.lock().unwrap().observers.on_error(error);
    }

    fn on_completed(&mut self) {
        &mut self.data.lock().unwrap().observers.on_completed();
    }
}

impl<Subscription, Item, Error> core::SharedSubject<Subscription, Item, Error>
    for PublishSubject<Subscription, Item, Error>
where
    Item: Copy + Send + Sync + 'static,
    Error: Copy + Send + Sync + 'static,
{
}

impl<Subscription, Item, Error> core::SharedObservable for PublishSubject<Subscription, Item, Error>
where
    Item: Send + Sync + 'static,
    Error: Send + Sync + 'static,
{
    type Subscription = core::SharedSubscription;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer:
            core::Observer<Self::Subscription, Self::Item, Self::Error> + Send + Sync + 'static,
    {
        self.data
            .lock()
            .unwrap()
            .observers
            .push(Box::new(consumer::shared::AutoOnSubscribe::new(observer)))
    }
}

impl<Subscription, Item, Error> core::Observable for PublishSubject<Subscription, Item, Error> {
    type Item = Item;
    type Error = Error;
}
