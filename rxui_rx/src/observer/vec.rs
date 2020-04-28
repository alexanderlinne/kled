use crate::core;
use crate::util::distribute_value;

impl<'o, Subscription, Item, Error> core::Observer<Subscription, Item, Error>
    for Vec<Box<dyn core::Observer<Subscription, Item, Error> + 'o>>
where
    Subscription: Copy,
    Item: Copy,
    Error: Copy,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        distribute_value(self, |o, s| o.on_subscribe(s), subscription);
    }

    fn on_next(&mut self, item: Item) {
        distribute_value(self, |o, i| o.on_next(i), item);
    }

    fn on_error(&mut self, error: Error) {
        distribute_value(self, |o, e| o.on_error(e), error);
    }

    fn on_completed(&mut self) {
        self.iter_mut().for_each(|o| o.on_completed());
    }
}

impl<Subscription, Item, Error> core::Observer<Subscription, Item, Error>
    for Vec<Box<dyn core::Observer<Subscription, Item, Error> + Send + Sync + 'static>>
where
    Subscription: Copy,
    Item: Copy,
    Error: Copy,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        distribute_value(self, |o, s| o.on_subscribe(s), subscription);
    }

    fn on_next(&mut self, item: Item) {
        distribute_value(self, |o, i| o.on_next(i), item);
    }

    fn on_error(&mut self, error: Error) {
        distribute_value(self, |o, e| o.on_error(e), error);
    }

    fn on_completed(&mut self) {
        self.iter_mut().for_each(|o| o.on_completed());
    }
}
