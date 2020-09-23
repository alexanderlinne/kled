use crate::core;
use crate::flow;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(new, reactive_operator)]
pub struct FlowOnBackpressureError<'o, Flow>
where
    Flow: core::LocalFlow<'o>,
{
    #[upstream(
        downstream = "OnBackpressureErrorSubscriber",
        subscription = "OnBackpressureErrorSubscription<Flow::Subscription, Flow::Error>"
    )]
    flow: Flow,
    #[reactive_operator(ignore)]
    phantom: PhantomData<&'o Self>,
}

pub struct OnBackpressureErrorSubscriber<Subscription, Subscriber, Item, Error> {
    subscriber: Option<Subscriber>,
    requested: Rc<RefCell<usize>>,
    phantom: PhantomData<(Subscription, Item, Error)>,
}

impl<'o, Subscription, Subscriber, Item, Error>
    OnBackpressureErrorSubscriber<Subscription, Subscriber, Item, Error>
where
    Subscriber:
        core::Subscriber<OnBackpressureErrorSubscription<Subscription, Error>, Item, Error> + 'o,
{
    pub fn new(subscriber: Subscriber) -> Self {
        Self {
            subscriber: Some(subscriber),
            requested: Rc::new(RefCell::new(0)),
            phantom: PhantomData,
        }
    }
}

impl<'o, Subscription, Subscriber, Item, Error> core::Subscriber<Subscription, Item, Error>
    for OnBackpressureErrorSubscriber<Subscription, Subscriber, Item, Error>
where
    Subscriber:
        core::Subscriber<OnBackpressureErrorSubscription<Subscription, Error>, Item, Error> + 'o,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        let requested = self.requested.clone();
        if let Some(subscriber) = self.subscriber.as_mut() {
            subscriber.on_subscribe(OnBackpressureErrorSubscription::new(
                subscription,
                requested,
            ))
        };
    }

    fn on_next(&mut self, item: Item) {
        if let Some(ref mut subscriber) = self.subscriber {
            if *self.requested.borrow() > 0 {
                subscriber.on_next(item);
                *self.requested.borrow_mut() -= 1;
            } else {
                subscriber.on_error(flow::Error::MissingBackpressure);
                self.subscriber = None
            }
        }
    }

    fn on_error(&mut self, error: flow::Error<Error>) {
        if let Some(ref mut subscriber) = self.subscriber {
            subscriber.on_error(error);
        }
    }

    fn on_completed(&mut self) {
        if let Some(ref mut subscriber) = self.subscriber {
            subscriber.on_completed();
        }
    }
}

#[derive(new)]
pub struct OnBackpressureErrorSubscription<Upstream, Error> {
    upstream: Upstream,
    requested: Rc<RefCell<usize>>,
    phantom: PhantomData<Error>,
}

impl<'o, Upstream, Error> core::Subscription for OnBackpressureErrorSubscription<Upstream, Error>
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
        *self.requested.borrow_mut() += count;
    }
}

#[cfg(test)]
mod tests {
    use crate::flow::local::*;
    use crate::prelude::*;
    use crate::subscriber::local::*;

    #[test]
    fn missing_backpressure() {
        let test_subscriber = TestSubscriber::default();
        vec![0, 1, 2]
            .into_flow()
            .on_backpressure_error()
            .subscribe(test_subscriber.clone());
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.items(), vec![]);
        matches!(
            test_subscriber.error(),
            Some(flow::Error::MissingBackpressure)
        );
    }

    #[test]
    fn upstream_error() {
        let test_subscriber = TestSubscriber::new(1);
        let test_flow = TestFlow::default();
        test_flow
            .clone()
            .on_backpressure_error()
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_error(());
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.items(), vec![0]);
        assert_eq!(test_subscriber.error(), Some(flow::Error::Upstream(())));
    }

    #[test]
    fn basic() {
        let test_subscriber = TestSubscriber::new(1);
        let test_flow = TestFlow::default().annotate_error_type(());
        test_flow
            .clone()
            .on_backpressure_error()
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_completed();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0]);
        assert_eq!(test_subscriber.error(), None);
    }
}
