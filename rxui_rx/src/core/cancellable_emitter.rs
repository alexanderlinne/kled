use crate::core;
use crate::util::distribute_value;

pub trait CancellableEmitter<Item, Error>: core::Emitter<Item, Error> {
    fn is_cancelled(&self) -> bool;
}

impl<'o, Item, Error> core::Emitter<Item, Error> for Box<dyn CancellableEmitter<Item, Error> + 'o> {
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

impl<'o, Item, Error> CancellableEmitter<Item, Error>
    for Box<dyn CancellableEmitter<Item, Error> + 'o>
{
    fn is_cancelled(&self) -> bool {
        (&**self).is_cancelled()
    }
}

impl<Item, Error> core::Emitter<Item, Error>
    for Box<dyn CancellableEmitter<Item, Error> + Send + 'static>
{
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

impl<Item, Error> CancellableEmitter<Item, Error>
    for Box<dyn CancellableEmitter<Item, Error> + Send + 'static>
{
    fn is_cancelled(&self) -> bool {
        (&**self).is_cancelled()
    }
}

impl<'o, Item, Error> core::Emitter<Item, Error>
    for Vec<Box<dyn CancellableEmitter<Item, Error> + 'o>>
where
    Item: Clone,
    Error: Clone,
{
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

impl<'o, Item, Error> core::CancellableEmitter<Item, Error>
    for Vec<Box<dyn core::CancellableEmitter<Item, Error> + 'o>>
where
    Item: Clone,
    Error: Clone,
{
    fn is_cancelled(&self) -> bool {
        self.iter().fold(true, |is_cancelled, item| {
            is_cancelled && item.is_cancelled()
        })
    }
}

impl<Item, Error> core::Emitter<Item, Error>
    for Vec<Box<dyn CancellableEmitter<Item, Error> + Send + 'static>>
where
    Item: Clone,
    Error: Clone,
{
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

impl<'o, Item, Error> core::CancellableEmitter<Item, Error>
    for Vec<Box<dyn core::CancellableEmitter<Item, Error> + Send + 'static>>
where
    Item: Clone,
    Error: Clone,
{
    fn is_cancelled(&self) -> bool {
        self.iter().fold(true, |is_cancelled, item| {
            is_cancelled && item.is_cancelled()
        })
    }
}
