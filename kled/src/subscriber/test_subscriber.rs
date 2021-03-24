use crate::core;
use crate::flow;
use async_std::sync::{Mutex, MutexGuardArc};
use async_trait::async_trait;
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
#[chronobreak]
use std::sync::Arc;
use thread_local::ThreadLocal;

struct ReentrantMutex<T: Send> {
    mutex: Arc<Mutex<T>>,
    lock: ThreadLocal<UnsafeCell<Option<MutexGuardArc<T>>>>,
}

unsafe impl<T: Send> Send for ReentrantMutex<T> {}
unsafe impl<T: Send> Sync for ReentrantMutex<T> {}

impl<T: Send> ReentrantMutex<T> {
    pub fn new(data: T) -> Self {
        Self {
            mutex: Arc::new(Mutex::new(data)),
            lock: ThreadLocal::new(),
        }
    }

    pub async fn lock(&'_ self) -> ReentrantMutexGuard<'_, T> {
        let has_lock = unsafe { &*self.lock.get_or_default().get() }.is_some();
        if !has_lock {
            unsafe { *self.lock.get_or_default().get() = Some(self.mutex.lock_arc().await) };
        }
        ReentrantMutexGuard {
            aquired_lock: !has_lock,
            mutex: self,
        }
    }
}

struct ReentrantMutexGuard<'a, T: Send> {
    aquired_lock: bool,
    mutex: &'a ReentrantMutex<T>,
}

unsafe impl<T: Send> Send for ReentrantMutexGuard<'_, T> {}
unsafe impl<T: Send> Sync for ReentrantMutexGuard<'_, T> {}

impl<T: Send> Drop for ReentrantMutexGuard<'_, T> {
    fn drop(&mut self) {
        if self.aquired_lock {
            unsafe { *self.mutex.lock.get().unwrap().get() = None };
        }
    }
}

impl<T: Send> Deref for ReentrantMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        let lock = self.mutex.lock.get().unwrap();
        unsafe { &*lock.get() }.as_ref().unwrap().deref()
    }
}

impl<T: Send> DerefMut for ReentrantMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        let lock = self.mutex.lock.get().unwrap();
        unsafe { &mut *lock.get() }.as_mut().unwrap().deref_mut()
    }
}

pub struct TestSubscriber<Subscription, Item, Error>
where
    Subscription: Send + Sync,
{
    subscription: Arc<ReentrantMutex<Option<Subscription>>>,
    data: Arc<Mutex<Data<Item, Error>>>,
}

struct Data<Item, Error> {
    items: Vec<Item>,
    error: Option<flow::Error<Error>>,
    is_completed: bool,
    request_on_subscribe: usize,
    request_on_next: usize,
    execute_on_next: Option<Box<dyn FnOnce() + Send + 'static>>,
}

impl<Subscription, Item, Error> Default for TestSubscriber<Subscription, Item, Error>
where
    Subscription: Send + Sync,
{
    fn default() -> Self {
        Self::new(0)
    }
}

impl<Subscription, Item, Error> TestSubscriber<Subscription, Item, Error>
where
    Subscription: Send + Sync,
{
    pub fn new(request_on_subscribe: usize) -> Self {
        Self {
            subscription: Arc::new(ReentrantMutex::new(None)),
            data: Arc::new(Mutex::new(Data {
                items: vec![],
                error: None,
                is_completed: false,
                request_on_subscribe,
                request_on_next: 0,
                execute_on_next: None,
            })),
        }
    }
}

impl<Subscription, Item, Error> Clone for TestSubscriber<Subscription, Item, Error>
where
    Subscription: Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            subscription: self.subscription.clone(),
            data: self.data.clone(),
        }
    }
}

pub type SubscriberStatus = crate::util::DownstreamStatus;

impl<Subscription, Item, Error> TestSubscriber<Subscription, Item, Error>
where
    Subscription: core::Subscription + Send + Sync,
{
    pub async fn status(&self) -> SubscriberStatus {
        if self.is_cancelled().await {
            SubscriberStatus::Cancelled
        } else if !self.is_subscribed().await {
            SubscriberStatus::Unsubscribed
        } else if self.has_error().await {
            SubscriberStatus::Error
        } else if self.is_completed().await {
            SubscriberStatus::Completed
        } else {
            SubscriberStatus::Subscribed
        }
    }

    pub async fn is_subscribed(&self) -> bool {
        self.subscription.lock().await.is_some()
    }

    pub async fn cancel(&mut self) {
        assert!(self.is_subscribed().await);
        self.subscription.lock().await.as_ref().unwrap().cancel().await;
    }

    pub async fn request_direct(&self, count: usize) {
        assert_ne!(self.status().await, SubscriberStatus::Unsubscribed);
        assert_ne!(self.status().await, SubscriberStatus::Cancelled);
        self.subscription
            .lock()
            .await
            .as_ref()
            .unwrap()
            .request(count)
            .await
    }

    pub async fn request_on_next(&mut self, count: usize) {
        self.data.lock().await.request_on_next += count;
    }

    pub async fn execute_on_next<Fn>(&mut self, f: Fn)
    where
        Fn: FnOnce() + Send + 'static,
    {
        self.data.lock().await.execute_on_next = Some(Box::new(f));
    }

    pub async fn has_error(&self) -> bool {
        self.data.lock().await.error.is_some()
    }

    pub async fn is_cancelled(&self) -> bool {
        let subscription = self.subscription.lock().await;
        if let Some(subscription) = subscription.as_ref() {
            subscription.is_cancelled().await
        } else {
            false
        }
    }

    pub async fn is_completed(&self) -> bool {
        self.data.lock().await.is_completed
    }
}

impl<Subscription, Item, Error> TestSubscriber<Subscription, Item, Error>
where
    Subscription: Send + Sync,
    Item: Clone,
    Error: Clone,
{
    pub async fn items(&self) -> Vec<Item> {
        self.data.lock().await.items.clone()
    }

    pub async fn error(&self) -> Option<flow::Error<Error>> {
        self.data.lock().await.error.clone()
    }
}

#[async_trait]
impl<Subscription, Item, Error> core::Subscriber<Subscription, Item, Error>
    for TestSubscriber<Subscription, Item, Error>
where
    Subscription: core::Subscription + Send + Sync,
    Item: Send,
    Error: Send,
{
    async fn on_subscribe(&mut self, subscription: Subscription) {
        assert_eq!(self.status().await, SubscriberStatus::Unsubscribed);
        subscription.request(self.data.lock().await.request_on_subscribe).await;
        *self.subscription.lock().await = Some(subscription);
    }

    async fn on_next(&mut self, item: Item) {
        assert_eq!(self.status().await, SubscriberStatus::Subscribed);
        let mut data = self.data.lock().await;
        data.items.push(item);
        if let Some(f) = data.execute_on_next.take() {
            f()
        };
        self.subscription
            .lock()
            .await
            .as_ref()
            .unwrap()
            .request(data.request_on_next).await;
        data.request_on_next = 0;
    }

    async fn on_error(&mut self, error: flow::Error<Error>) {
        assert_eq!(self.status().await, SubscriberStatus::Subscribed);
        self.data.lock().await.error = Some(error)
    }

    async fn on_completed(&mut self) {
        assert_eq!(self.status().await, SubscriberStatus::Subscribed);
        self.data.lock().await.is_completed = true;
    }
}
