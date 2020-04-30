use crate::core;

pub trait CancellableConsumer<Item, Error>: core::Consumer<Item, Error> {
    fn is_cancelled(&self) -> bool;
}

impl<'o, Item, Error> core::Consumer<Item, Error>
    for Box<dyn CancellableConsumer<Item, Error> + 'o>
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

impl<'o, Item, Error> CancellableConsumer<Item, Error>
    for Box<dyn CancellableConsumer<Item, Error> + 'o>
{
    fn is_cancelled(&self) -> bool {
        (&**self).is_cancelled()
    }
}

impl<Item, Error> core::Consumer<Item, Error>
    for Box<dyn CancellableConsumer<Item, Error> + Send + 'static>
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

impl<Item, Error> CancellableConsumer<Item, Error>
    for Box<dyn CancellableConsumer<Item, Error> + Send + 'static>
{
    fn is_cancelled(&self) -> bool {
        (&**self).is_cancelled()
    }
}
