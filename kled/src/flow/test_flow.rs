use crate::core;
use crate::flow;
use crate::subscription::*;
use async_std::sync::Mutex;
use async_trait::async_trait;
#[chronobreak]
use std::sync::Arc;

#[derive(Clone)]
pub struct TestFlow<Item, Error> {
    data: Arc<Mutex<Data<Item, Error>>>,
}

struct Data<Item, Error> {
    emitter: Option<flow::BoxEmitter<Item, Error>>,
}

impl<Item, Error> Default for TestFlow<Item, Error> {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(Data { emitter: None })),
        }
    }
}

impl<Item, Error> TestFlow<Item, Error> {
    pub async fn has_observer(&self) -> bool {
        self.data.lock().await.emitter.is_some()
    }

    pub fn annotate_item_type(self, _: Item) -> Self {
        self
    }

    pub fn annotate_error_type(self, _: Error) -> Self {
        self
    }

    pub async fn is_cancelled(&self) -> bool {
        assert!(self.has_observer().await);
        match self.data.lock().await.emitter {
            Some(ref consumer) => consumer.is_cancelled(),
            None => panic!(),
        }
    }

    pub async fn emit(&self, item: Item) {
        assert!(self.has_observer().await);
        match self.data.lock().await.emitter {
            Some(ref mut consumer) => consumer.on_next(item).await,
            None => panic!(),
        }
    }

    pub async fn emit_all<IntoIter>(&self, into_iter: IntoIter)
    where
        IntoIter: IntoIterator<Item = Item>,
    {
        for value in into_iter.into_iter() {
            self.emit(value).await;
        }
    }

    pub async fn emit_error(&self, error: Error) {
        assert!(self.has_observer().await);
        let emitter = &mut self.data.lock().await.emitter;
        match emitter {
            Some(ref mut consumer) => consumer.on_error(error).await,
            None => panic!(),
        }
        *emitter = None;
    }

    pub async fn emit_completed(&self) {
        assert!(self.has_observer().await);
        let emitter = &mut self.data.lock().await.emitter;
        match emitter {
            Some(ref mut consumer) => consumer.on_completed().await,
            None => panic!(),
        }
        *emitter = None;
    }
}

#[async_trait]
impl<Item, Error> core::Flow<ArcSubscription, Item, Error> for TestFlow<Item, Error>
where
    Item: Send + 'static,
    Error: Send + 'static,
{
    async fn subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<ArcSubscription, Item, Error> + Send + 'static,
    {
        assert!(!self.has_observer().await);
        let mut data = self.data.lock().await;
        data.emitter = Some(flow::BoxEmitter::from(subscriber).await);
    }
}
