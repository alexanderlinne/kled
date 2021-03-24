use crate::cancellable::*;
use crate::core;
use crate::observable;
use async_std::sync::Mutex;
use async_trait::async_trait;
#[chronobreak]
use std::sync::Arc;

#[derive(Clone)]
pub struct TestObservable<Item, Error> {
    data: Arc<Mutex<Data<Item, Error>>>,
}

struct Data<Item, Error> {
    emitter: Option<observable::BoxEmitter<Item, Error>>,
}

impl<Item, Error> Default for TestObservable<Item, Error> {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(Data { emitter: None })),
        }
    }
}

impl<Item, Error> TestObservable<Item, Error> {
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

    pub async fn emit_on_completed(&self) {
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
impl<Item, Error> core::Observable<ArcCancellable, Item, Error> for TestObservable<Item, Error>
where
    Item: Send + 'static,
    Error: Send + 'static,
{
    async fn subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<ArcCancellable, Item, Error> + Send + 'static,
    {
        assert!(!self.has_observer().await);
        let mut lock = self.data.lock().await;
        lock.emitter = Some(observable::BoxEmitter::from(observer).await);
    }
}
