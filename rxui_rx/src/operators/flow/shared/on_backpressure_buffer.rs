use crate::core;
use crate::flow;
use crate::marker;
use crossbeam::channel::{bounded, Receiver, Sender};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};

#[derive(new, reactive_operator)]
pub struct FlowOnBackpressureBuffer<Flow>
where
    Flow: core::SharedFlow,
    Flow::Item: Send,
    Flow::Error: Send,
{
    #[upstream(
        operator = "on_backpressure_buffer_with_capacity",
        subscription = "OnBackpressureBufferSubscription<Flow::Subscription, Flow::Item, Flow::Error>"
    )]
    flow: Flow,
    buffer_strategy: flow::BufferStrategy,
    buffer_capacity: usize,
}

impl<Flow> marker::Shared<marker::Flow<Flow>>
where
    Flow: core::Flow,
    Flow::Item: Send,
    Flow::Error: Send,
{
    pub fn on_backpressure_buffer(
        self,
        buffer_strategy: flow::BufferStrategy,
    ) -> marker::Shared<marker::Flow<FlowOnBackpressureBuffer<Flow>>>
    where
        Flow: core::SharedFlow + Sized,
    {
        marker::Shared::new(marker::Flow::new(FlowOnBackpressureBuffer::new(
            self.actual.actual,
            buffer_strategy,
            flow::default_buffer_capacity(),
        )))
    }
}

pub struct OnBackpressureBufferSubscriber<Subscription, Item, Error> {
    data: Arc<Data<Subscription, Item, Error>>,
    buffer_strategy: flow::BufferStrategy,
}

type BoxedSubscriber<Subscription, Item, Error> = Box<
    dyn core::Subscriber<OnBackpressureBufferSubscription<Subscription, Item, Error>, Item, Error>
        + Send
        + 'static,
>;

pub struct Data<Subscription, Item, Error> {
    subscriber: Mutex<Option<BoxedSubscriber<Subscription, Item, Error>>>,
    requested: AtomicUsize,
    channel: (Sender<Item>, Receiver<Item>),
}

impl<Subscription, Item, Error> OnBackpressureBufferSubscriber<Subscription, Item, Error>
where
    Item: Send + 'static,
{
    pub fn new<Subscriber>(
        subscriber: Subscriber,
        buffer_strategy: flow::BufferStrategy,
        buffer_capacity: usize,
    ) -> Self
    where
        Subscriber: core::Subscriber<
                OnBackpressureBufferSubscription<Subscription, Item, Error>,
                Item,
                Error,
            > + Send
            + 'static,
    {
        let data = Arc::new(Data {
            subscriber: Mutex::new(Some(Box::new(subscriber))),
            requested: AtomicUsize::default(),
            channel: bounded(buffer_capacity),
        });
        Self {
            data,
            buffer_strategy,
        }
    }

    fn add_to_queue(&self, item: Item) {
        if !self.data.channel.0.is_full() {
            self.data.channel.0.send(item).unwrap();
        } else {
            use flow::BufferStrategy::*;
            match self.buffer_strategy {
                Error => {
                    if let Some(mut subscriber) = self.data.subscriber.lock().unwrap().take() {
                        subscriber.on_error(flow::Error::MissingBackpressure)
                    };
                }
                DropOldest => {
                    self.data.channel.1.recv().unwrap();
                    self.data.channel.0.send(item).unwrap();
                }
                DropLatest => {}
            }
        }
    }
}

fn drain<Subscription, Item, Error>(
    data: &Arc<Data<Subscription, Item, Error>>,
    subscriber: &mut BoxedSubscriber<Subscription, Item, Error>,
    mut requested: usize,
) {
    let mut emitted = 0;
    while emitted < requested {
        let item = if let Ok(item) = data.channel.1.recv() {
            item
        } else {
            break;
        };
        subscriber.on_next(item);
        emitted += 1;
        // If the loop would finish, update the count of requested items as
        // on_next may have called request
        if emitted == requested {
            requested = data.requested.load(Ordering::Relaxed);
        }
    }
    data.requested.store(requested - emitted, Ordering::Relaxed);
}

impl<Subscription, Item, Error> core::Subscriber<Subscription, Item, Error>
    for OnBackpressureBufferSubscriber<Subscription, Item, Error>
where
    Item: Send + 'static,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        let data = Arc::downgrade(&self.data);
        if let Some(subscriber) = self.data.subscriber.lock().unwrap().as_mut() {
            subscriber.on_subscribe(OnBackpressureBufferSubscription::new(subscription, data))
        };
    }

    fn on_next(&mut self, item: Item) {
        let requested = self.data.requested.load(Ordering::Relaxed);
        self.add_to_queue(item);
        if requested > 0 {
            if let Some(ref mut subscriber) = *self.data.subscriber.lock().unwrap() {
                drain(&self.data, subscriber, requested);
            }
        }
    }

    fn on_error(&mut self, error: flow::Error<Error>) {
        if let Some(subscriber) = self.data.subscriber.lock().unwrap().as_mut() {
            subscriber.on_error(error)
        };
    }

    fn on_completed(&mut self) {
        if let Some(subscriber) = self.data.subscriber.lock().unwrap().as_mut() {
            subscriber.on_completed()
        };
    }
}

#[derive(new)]
pub struct OnBackpressureBufferSubscription<Upstream, Item, Error> {
    upstream: Upstream,
    data: Weak<Data<Upstream, Item, Error>>,
}

unsafe impl<Upstream, Item, Error> Sync
    for OnBackpressureBufferSubscription<Upstream, Item, Error>
{
}

impl<'o, Upstream, Item, Error> core::Subscription
    for OnBackpressureBufferSubscription<Upstream, Item, Error>
where
    Upstream: core::Subscription,
{
    fn cancel(&self) {
        self.upstream.cancel()
    }

    fn is_cancelled(&self) -> bool {
        self.upstream.is_cancelled()
    }

    fn request(&self, count: usize) {
        let data = match self.data.upgrade() {
            None => return,
            Some(data) => data,
        };

        let requested = data.requested.fetch_add(count, Ordering::Relaxed) + count;
        if requested > 0 {
            // This prevents more than one reentrant call of request is the
            // subscriber is borrowed mutably either here or in on_next
            if let Ok(mut subscriber) = data.subscriber.try_lock() {
                if let Some(mut subscriber) = (&mut *subscriber).as_mut() {
                    drain(&data, &mut subscriber, requested)
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::flow::shared::*;
    use crate::prelude::*;
    use crate::subscriber::shared::*;

    #[test]
    fn basic() {
        let mut test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default().annotate_error_type(());
        test_flow
            .clone()
            .on_backpressure_buffer_with_capacity(flow::BufferStrategy::Error, 5)
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit(1);
        test_subscriber.request_direct(1);
        test_flow.emit(2);
        test_subscriber.request_on_next(1);
        test_flow.emit(3);
        test_subscriber.request_direct(1);
        test_flow.emit(4);
        test_flow.emit_completed();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0, 1, 2]);
    }

    #[test]
    fn upstream_error() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default();
        test_flow
            .clone()
            .on_backpressure_buffer_with_capacity(flow::BufferStrategy::Error, 1)
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_error(());
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.items(), vec![]);
    }

    #[test]
    fn error_strategy() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default().annotate_error_type(());
        test_flow
            .clone()
            .on_backpressure_buffer_with_capacity(flow::BufferStrategy::Error, 1)
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit(1);
        test_subscriber.request_direct(1);
        test_flow.emit_completed();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.items(), vec![]);
    }

    #[test]
    fn drop_oldest_strategy() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default().annotate_error_type(());
        test_flow
            .clone()
            .on_backpressure_buffer_with_capacity(flow::BufferStrategy::DropOldest, 1)
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit(1);
        test_subscriber.request_direct(1);
        test_flow.emit_completed();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![1]);
    }

    #[test]
    fn drop_latest_strategy() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default().annotate_error_type(());
        test_flow
            .clone()
            .on_backpressure_buffer_with_capacity(flow::BufferStrategy::DropLatest, 1)
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit(1);
        test_subscriber.request_direct(1);
        test_flow.emit_completed();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0]);
    }
}
