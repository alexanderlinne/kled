use crate::core;

pub trait Subscriber<Item, Error> {
    fn on_subscribe(&mut self, subscription: Box<dyn core::flow::Subscription>);
    fn on_next(&mut self, item: Item);
    fn on_error(&mut self, errpr: Error);
    fn on_completed(&mut self);
}

impl<Item, Error> Subscriber<Item, Error> for Box<dyn Subscriber<Item, Error> + Send + Sync + 'static> {
    fn on_subscribe(&mut self, subscription: Box<dyn core::flow::Subscription>) {
        (&mut **self).on_subscribe(subscription)
    }

    fn on_next(&mut self, item: Item) {
        (&mut **self).on_next(item)
    }

    fn on_error(&mut self, error: Error) {
        (&mut **self).on_error(error)
    }

    fn on_completed(&mut self) {
        (&mut **self).on_completed()
    }
}

pub trait UnsubscribableSubscribe<Item, Error>: Subscriber<Item, Error> {
    fn is_unsubscribed(&self) -> bool;
}
