use crate::core;

pub trait Subscriber<Subscription, Item, Error> {
    fn on_subscribe(&mut self, subscription: Subscription);
    fn on_next(&mut self, item: Item);
    fn on_error(&mut self, error: core::FlowError<Error>);
    fn on_completed(&mut self);
}

impl<'o, Subscription, Item, Error> Subscriber<Subscription, Item, Error>
    for Box<dyn Subscriber<Subscription, Item, Error> + 'o>
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        (&mut **self).on_subscribe(subscription)
    }

    fn on_next(&mut self, item: Item) {
        (&mut **self).on_next(item)
    }

    fn on_error(&mut self, error: core::FlowError<Error>) {
        (&mut **self).on_error(error)
    }

    fn on_completed(&mut self) {
        (&mut **self).on_completed()
    }
}

impl<Subscription, Item, Error> Subscriber<Subscription, Item, Error>
    for Box<dyn Subscriber<Subscription, Item, Error> + Send + 'static>
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        (&mut **self).on_subscribe(subscription)
    }

    fn on_next(&mut self, item: Item) {
        (&mut **self).on_next(item)
    }

    fn on_error(&mut self, error: core::FlowError<Error>) {
        (&mut **self).on_error(error)
    }

    fn on_completed(&mut self) {
        (&mut **self).on_completed()
    }
}
