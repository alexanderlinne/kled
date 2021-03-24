use crate::core;
use async_std::sync::Mutex;
use async_trait::async_trait;
#[chronobreak]
use std::sync::Arc;

pub struct LazyCancellableStub<Cancellable> {
    data: Arc<Mutex<Data<Cancellable>>>,
}

impl<Cancellable> Default for LazyCancellableStub<Cancellable> {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(Data::default())),
        }
    }
}

impl<Cancellable> LazyCancellableStub<Cancellable>
where
    Cancellable: core::Cancellable,
{
    pub fn cancellable(&self) -> LazyCancellable<Cancellable> {
        LazyCancellable {
            data: self.data.clone(),
        }
    }

    pub async fn set_cancellable(&mut self, cancellable: Cancellable) {
        let mut data = self.data.lock().await;
        if data.cancelled {
            cancellable.cancel().await;
        }
        data.subscription = Some(cancellable);
    }
}

#[derive(Clone)]
pub struct LazyCancellable<Cancellable> {
    data: Arc<Mutex<Data<Cancellable>>>,
}

#[async_trait]
impl<Cancellable> core::Cancellable for LazyCancellable<Cancellable>
where
    Cancellable: core::Cancellable + Send + Sync,
{
    async fn cancel(&self) {
        let mut data = self.data.lock().await;
        if let Some(subscription) = &data.subscription {
            subscription.cancel().await;
        } else {
            data.cancelled = true;
        }
    }
}

struct Data<Cancellable> {
    cancelled: bool,
    subscription: Option<Cancellable>,
}

impl<Cancellable> Default for Data<Cancellable> {
    fn default() -> Self {
        Self {
            cancelled: false,
            subscription: None,
        }
    }
}
