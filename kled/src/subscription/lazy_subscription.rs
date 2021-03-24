use crate::core;
use async_std::sync::Mutex;
use async_trait::async_trait;
#[chronobreak]
use std::sync::Arc;

pub struct LazySubscriptionStub<Subscription> {
    data: Arc<Mutex<Data<Subscription>>>,
}

impl<Subscription> Default for LazySubscriptionStub<Subscription> {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(Data::default())),
        }
    }
}

impl<Subscription> LazySubscriptionStub<Subscription>
where
    Subscription: core::Subscription,
{
    pub fn subscription(&self) -> LazySubscription<Subscription> {
        LazySubscription {
            data: self.data.clone(),
        }
    }

    pub async fn set_subscription(&mut self, subscription: Subscription) {
        let mut data = self.data.lock().await;
        if data.cancelled {
            subscription.cancel().await;
        } else {
            subscription.request(data.requested).await;
        }
        data.subscription = Some(subscription);
    }
}

#[derive(Clone)]
pub struct LazySubscription<Subscription> {
    data: Arc<Mutex<Data<Subscription>>>,
}

#[async_trait]
impl<Subscription> core::Subscription for LazySubscription<Subscription>
where
    Subscription: core::Subscription + Send + Sync,
{
    async fn cancel(&self) {
        let mut data = self.data.lock().await;
        if let Some(subscription) = &data.subscription {
            subscription.cancel().await;
        } else {
            data.cancelled = true;
        }
    }

    async fn is_cancelled(&self) -> bool {
        let data = self.data.lock().await;
        if let Some(subscription) = &data.subscription {
            subscription.is_cancelled().await
        } else {
            data.cancelled
        }
    }

    async fn request(&self, count: usize) {
        let mut data = self.data.lock().await;
        if let Some(subscription) = &data.subscription {
            subscription.request(count).await;
        } else {
            data.requested += count;
        }
    }
}

struct Data<Subscription> {
    cancelled: bool,
    requested: usize,
    subscription: Option<Subscription>,
}

impl<Subscription> Default for Data<Subscription> {
    fn default() -> Self {
        Self {
            cancelled: false,
            requested: 0,
            subscription: None,
        }
    }
}
