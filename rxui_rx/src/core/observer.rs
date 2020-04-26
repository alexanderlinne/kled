use crate::core;

pub trait Observer<Item, Error> {
    fn on_subscribe(&mut self, subscription: Box<dyn core::observable::Subscription>);
    fn on_next(&mut self, item: Item);
    fn on_error(&mut self, errpr: Error);
    fn on_completed(&mut self);
}

impl<Item, Error> Observer<Item, Error>
    for Box<dyn Observer<Item, Error> + Send + Sync + 'static>
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
