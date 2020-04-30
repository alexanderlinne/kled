use crate::core;
use crate::util::distribute_value;

impl<'o, Item, Error> core::Consumer<Item, Error>
    for Vec<Box<dyn core::CancellableConsumer<Item, Error> + 'o>>
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

impl<'o, Item, Error> core::CancellableConsumer<Item, Error>
    for Vec<Box<dyn core::CancellableConsumer<Item, Error> + 'o>>
where
    Item: Clone,
    Error: Clone,
{
    fn is_cancelled(&self) -> bool {
        self.iter().fold(true, |is_unsubscribed, item| {
            is_unsubscribed && item.is_cancelled()
        })
    }
}

impl<Item, Error> core::Consumer<Item, Error>
    for Vec<Box<dyn core::CancellableConsumer<Item, Error> + Send + 'static>>
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

impl<'o, Item, Error> core::CancellableConsumer<Item, Error>
    for Vec<Box<dyn core::CancellableConsumer<Item, Error> + Send + 'static>>
where
    Item: Clone,
    Error: Clone,
{
    fn is_cancelled(&self) -> bool {
        self.iter().fold(true, |is_unsubscribed, item| {
            is_unsubscribed && item.is_cancelled()
        })
    }
}
