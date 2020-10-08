use crate::core;
use crate::sync::{Arc, Mutex};

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

impl<Subscription> core::SubscriptionProvider for LazySubscriptionStub<Subscription>
where
    Subscription: core::Subscription,
{
    type Subscription = LazySubscription<Subscription>;

    fn subscription(&self) -> LazySubscription<Subscription> {
        LazySubscription {
            data: self.data.clone(),
        }
    }
}

impl<Subscription> LazySubscriptionStub<Subscription>
where
    Subscription: core::Subscription,
{
    pub fn set_subscription(&mut self, subscription: Subscription) {
        let mut data = self.data.lock();
        if data.cancelled {
            subscription.cancel();
        } else {
            subscription.request(data.requested);
        }
        data.subscription = Some(subscription);
    }
}

#[derive(Clone)]
pub struct LazySubscription<Subscription> {
    data: Arc<Mutex<Data<Subscription>>>,
}

impl<Subscription> core::Subscription for LazySubscription<Subscription>
where
    Subscription: core::Subscription,
{
    fn cancel(&self) {
        let mut data = self.data.lock();
        if let Some(subscription) = &data.subscription {
            subscription.cancel();
        } else {
            data.cancelled = true;
        }
    }

    fn is_cancelled(&self) -> bool {
        let data = self.data.lock();
        if let Some(subscription) = &data.subscription {
            subscription.is_cancelled()
        } else {
            data.cancelled
        }
    }

    fn request(&self, count: usize) {
        let mut data = self.data.lock();
        if let Some(subscription) = &data.subscription {
            subscription.request(count);
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
