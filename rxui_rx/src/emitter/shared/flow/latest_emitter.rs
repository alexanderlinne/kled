use crate::core;
use crate::flow;
use crate::subscription::shared::*;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub struct LatestEmitter<Subscriber, Item, Error> {
    data: Arc<Data<Subscriber, Item>>,
    stub: LambdaSubscriptionStub,
    phantom: PhantomData<(Item, Error)>,
}

pub struct Data<Subscriber, Item> {
    subscriber: Mutex<Subscriber>,
    requested: AtomicUsize,
    latest: Mutex<Option<Item>>,
}

impl<Subscriber, Item, Error> LatestEmitter<Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<LambdaSubscription, Item, Error> + Send + 'static,
    Item: Send + 'static,
{
    pub fn new(subscriber: Subscriber) -> Self {
        let data = Arc::new(Data {
            subscriber: Mutex::new(subscriber),
            requested: AtomicUsize::default(),
            latest: Mutex::new(None),
        });
        let stub = Self::create_subscription(data.clone());
        data.subscriber
            .lock()
            .unwrap()
            .on_subscribe(stub.subscription());
        Self {
            data,
            stub,
            phantom: PhantomData,
        }
    }

    fn create_subscription(data: Arc<Data<Subscriber, Item>>) -> LambdaSubscriptionStub {
        LambdaSubscriptionStub::new(move |count: usize| {
            let requested = data.requested.fetch_add(count, Ordering::Relaxed) + count;
            if requested > 0 {
                if let Ok(mut subscriber) = data.subscriber.try_lock() {
                    let item = data.latest.lock().unwrap().take();
                    if let Some(item) = item {
                        subscriber.on_next(item);
                        data.requested.fetch_sub(1, Ordering::Relaxed);
                    }
                }
            }
        })
    }
}

impl<Subscriber, Item, Error> core::FlowEmitter<Item, Error>
    for LatestEmitter<Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<LambdaSubscription, Item, Error> + Send + 'static,
{
    fn on_next(&mut self, item: Item) {
        if self.data.requested.load(Ordering::Relaxed) > 0 {
            let mut subscriber = self.data.subscriber.lock().unwrap();
            *self.data.latest.lock().unwrap() = None;
            subscriber.on_next(item);
            self.data.requested.fetch_sub(1, Ordering::Relaxed);
        } else {
            *self.data.latest.lock().unwrap() = Some(item);
        }
    }

    fn on_error(&mut self, error: Error) {
        let mut subscriber = self.data.subscriber.lock().unwrap();
        *self.data.latest.lock().unwrap() = None;
        subscriber.on_error(flow::Error::Upstream(error));
    }

    fn on_completed(&mut self) {
        let mut subscriber = self.data.subscriber.lock().unwrap();
        *self.data.latest.lock().unwrap() = None;
        subscriber.on_completed();
    }

    fn is_cancelled(&self) -> bool {
        self.stub.is_cancelled()
    }
}
