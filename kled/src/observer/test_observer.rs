use crate::core;
use async_std::sync::Mutex;
use async_trait::async_trait;
#[chronobreak]
use std::sync::Arc;

#[derive(Clone)]
pub struct TestObserver<Cancellable, Item, Error> {
    data: Arc<Mutex<Data<Cancellable, Item, Error>>>,
}

struct Data<Cancellable, Item, Error> {
    cancellable: Option<Cancellable>,
    items: Vec<Item>,
    error: Option<Error>,
    is_completed: bool,
    is_cancelled: bool,
}

impl<Cancellable, Item, Error> Default for TestObserver<Cancellable, Item, Error> {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(Data {
                cancellable: None,
                items: vec![],
                error: None,
                is_completed: false,
                is_cancelled: false,
            })),
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

impl<Cancellable, Item, Error> TestObserver<Cancellable, Item, Error>
where
    Cancellable: core::Cancellable,
{
    pub async fn status(&self) -> ObserverStatus {
        if self.data.lock().await.is_cancelled {
            ObserverStatus::Cancelled
        } else if !self.is_subscribed().await {
            ObserverStatus::Unsubscribed
        } else if self.has_error().await {
            ObserverStatus::Error
        } else if self.is_completed().await {
            ObserverStatus::Completed
        } else {
            ObserverStatus::Subscribed
        }
    }

    pub async fn is_subscribed(&self) -> bool {
        self.data.lock().await.cancellable.is_some()
    }

    pub async fn cancel(&mut self) {
        assert!(self.is_subscribed().await);
        let mut data = self.data.lock().await;
        data.cancellable.take().unwrap().cancel().await;
        data.is_cancelled = true;
    }

    pub async fn has_error(&self) -> bool {
        self.data.lock().await.error.is_some()
    }

    pub async fn is_completed(&self) -> bool {
        self.data.lock().await.is_completed
    }
}

impl<Cancellable, Item, Error> TestObserver<Cancellable, Item, Error>
where
    Item: Clone,
    Error: Clone,
{
    pub async fn items(&self) -> Vec<Item> {
        self.data.lock().await.items.clone()
    }

    pub async fn error(&self) -> Option<Error> {
        self.data.lock().await.error.clone()
    }
}

#[async_trait]
impl<Cancellable, Item, Error> core::Observer<Cancellable, Item, Error>
    for TestObserver<Cancellable, Item, Error>
where
    Cancellable: core::Cancellable + Send,
    Item: Send,
    Error: Send,
{
    async fn on_subscribe(&mut self, cancellable: Cancellable) {
        assert_eq!(self.status().await, ObserverStatus::Unsubscribed);
        self.data.lock().await.cancellable = Some(cancellable);
    }

    async fn on_next(&mut self, item: Item) {
        assert_eq!(self.status().await, ObserverStatus::Subscribed);
        self.data.lock().await.items.push(item);
    }

    async fn on_error(&mut self, error: Error) {
        assert_eq!(self.status().await, ObserverStatus::Subscribed);
        self.data.lock().await.error = Some(error)
    }

    async fn on_completed(&mut self) {
        assert_eq!(self.status().await, ObserverStatus::Subscribed);
        self.data.lock().await.is_completed = true;
    }
}
