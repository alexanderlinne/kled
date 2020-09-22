use crate::core;
use crate::flow;
use crate::subscription::shared::*;

impl<Subscriber, Item, Error> core::IntoSharedFlowEmitter<Item, Error> for Subscriber
where
    Subscriber: core::Subscriber<LambdaSubscription, Item, Error> + Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
    type Emitter = Box<dyn core::FlowEmitter<Item, Error> + Send + 'static>;

    fn into_shared_emitter(self, strategy: flow::BackpressureStrategy) -> Self::Emitter {
        match strategy {
            flow::BackpressureStrategy::Missing => Box::new(super::flow::MissingEmitter::new(self)),
            flow::BackpressureStrategy::Error => Box::new(super::flow::ErrorEmitter::new(self)),
            flow::BackpressureStrategy::Drop => Box::new(super::flow::DropEmitter::new(self)),
            flow::BackpressureStrategy::Latest => Box::new(super::flow::LatestEmitter::new(self)),
            _ => unimplemented! {},
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::util::shared::*;

    #[test]
    fn missing() {
        let test_subscriber = TestSubscriber::new(1);
        let scheduler = scheduler::NewThreadScheduler::default();
        vec![0, 1, 2]
            .into_flow(flow::BackpressureStrategy::Missing)
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone());
        scheduler.join();
        assert_eq!(test_subscriber.status(), ObserverStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0, 1, 2]);
    }

    #[test]
    fn drop() {
        let test_subscriber = TestSubscriber::new(1);
        let scheduler = scheduler::NewThreadScheduler::default();
        vec![0, 1, 2]
            .into_flow(flow::BackpressureStrategy::Drop)
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone());
        scheduler.join();
        assert_eq!(test_subscriber.status(), ObserverStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0]);
    }

    #[test]
    fn drop_error() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::new(flow::BackpressureStrategy::Drop);
        test_flow
            .clone()
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_error(());
        scheduler.join();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.error(), Some(flow::Error::Upstream(())));
    }

    #[test]
    fn latest() {
        let mut test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::new(flow::BackpressureStrategy::Latest).annotate_error_type(());
        test_flow.clone().subscribe(test_subscriber.clone());
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
        assert_eq!(test_subscriber.items(), vec![1, 3, 4]);
    }

    #[test]
    fn latest_error() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::new(flow::BackpressureStrategy::Latest);
        test_flow.clone().subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_error(());
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.items(), vec![]);
    }

    #[test]
    fn error_missing_backpressure() {
        let test_subscriber = TestSubscriber::default();
        vec![0, 1, 2]
            .into_flow(flow::BackpressureStrategy::Error)
            .subscribe(test_subscriber.clone());
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.items(), vec![]);
        matches!(
            test_subscriber.error(),
            Some(flow::Error::MissingBackpressure)
        );
    }

    #[test]
    fn error_upstream_error() {
        let test_subscriber = TestSubscriber::new(1);
        let test_flow = TestFlow::new(flow::BackpressureStrategy::Error);
        test_flow.clone().subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_error(());
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.items(), vec![0]);
        assert_eq!(test_subscriber.error(), Some(flow::Error::Upstream(())));
    }

    #[test]
    fn error_completed() {
        let test_subscriber = TestSubscriber::new(1);
        let test_flow = TestFlow::new(flow::BackpressureStrategy::Error).annotate_error_type(());
        test_flow.clone().subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_completed();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0]);
        assert_eq!(test_subscriber.error(), None);
    }
}
