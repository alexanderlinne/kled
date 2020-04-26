use crate::core;

pub trait Subscriber<Item, Error>: core::Observer<Item, Error> {
    fn is_unsubscribed(&self) -> bool;
}

impl<Item, Error> core::Observer<Item, Error>
    for Box<dyn Subscriber<Item, Error>>
{
    fn on_subscribe(&mut self, subscription: Box<dyn core::observable::Subscription>) {
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

impl<Item, Error> Subscriber<Item, Error>
    for Box<dyn Subscriber<Item, Error>>
{
    fn is_unsubscribed(&self) -> bool {
        (&**self).is_unsubscribed()
    }
}
