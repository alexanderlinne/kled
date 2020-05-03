use crate::core;
use crate::util::distribute_value;

impl<'o, Cancellable, Item, Error> core::Observer<Cancellable, Item, Error>
    for Vec<Box<dyn core::Observer<Cancellable, Item, Error> + 'o>>
where
    Cancellable: Copy,
    Item: Copy,
    Error: Copy,
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        distribute_value(self, |o, s| o.on_subscribe(s), cancellable);
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

impl<Cancellable, Item, Error> core::Observer<Cancellable, Item, Error>
    for Vec<Box<dyn core::Observer<Cancellable, Item, Error> + Send + 'static>>
where
    Cancellable: Copy,
    Item: Copy,
    Error: Copy,
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        distribute_value(self, |o, s| o.on_subscribe(s), cancellable);
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
