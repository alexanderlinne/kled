use crate::core;
use crate::flow;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(new, reactive_operator)]
pub struct FlowOnBackpressureDrop<'o, Flow>
where
    Flow: core::LocalFlow<'o>,
{
    #[upstream(subscription = "OnBackpressureDropSubscription<Flow::Subscription, Flow::Error>")]
    flow: Flow,
    #[reactive_operator(ignore)]
    phantom: PhantomData<&'o Self>,
}

pub struct OnBackpressureDropSubscriber<Subscription, Subscriber, Item, Error> {
    subscriber: Subscriber,
    requested: Rc<RefCell<usize>>,
    phantom: PhantomData<(Subscription, Item, Error)>,
}

impl<'o, Subscription, Subscriber, Item, Error>
    OnBackpressureDropSubscriber<Subscription, Subscriber, Item, Error>
where
    Subscriber:
        core::Subscriber<OnBackpressureDropSubscription<Subscription, Error>, Item, Error> + 'o,
{
    pub fn new(subscriber: Subscriber) -> Self {
        Self {
            subscriber,
            requested: Rc::new(RefCell::new(0)),
            phantom: PhantomData,
        }
    }
}

impl<'o, Subscription, Subscriber, Item, Error> core::Subscriber<Subscription, Item, Error>
    for OnBackpressureDropSubscriber<Subscription, Subscriber, Item, Error>
where
    Subscriber:
        core::Subscriber<OnBackpressureDropSubscription<Subscription, Error>, Item, Error> + 'o,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        let requested = self.requested.clone();
        self.subscriber
            .on_subscribe(OnBackpressureDropSubscription::new(subscription, requested));
    }

    fn on_next(&mut self, item: Item) {
        if *self.requested.borrow() > 0 {
            self.subscriber.on_next(item);
            *self.requested.borrow_mut() -= 1;
        }
    }

    fn on_error(&mut self, error: flow::Error<Error>) {
        self.subscriber.on_error(error);
    }

    fn on_completed(&mut self) {
        self.subscriber.on_completed();
    }
}

#[derive(new)]
pub struct OnBackpressureDropSubscription<Upstream, Error> {
    upstream: Upstream,
    requested: Rc<RefCell<usize>>,
    phantom: PhantomData<Error>,
}

impl<'o, Upstream, Error> core::Subscription for OnBackpressureDropSubscription<Upstream, Error>
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
    fn drop() {
        let test_subscriber = TestSubscriber::new(1);
        vec![0, 1, 2]
            .into_flow()
            .on_backpressure_drop()
            .subscribe(test_subscriber.clone());
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0]);
    }

    #[test]
    fn drop_error() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default();
        test_flow
            .clone()
            .on_backpressure_drop()
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_error(());
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.items(), vec![]);
        assert_eq!(test_subscriber.error(), Some(flow::Error::Upstream(())));
    }
}
