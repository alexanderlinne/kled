use crate::core;

pub trait UnsubscribableEmitter<Item, Error>: core::Emitter<Item, Error> {
    fn is_unsubscribed(&self) -> bool;
}

impl<'o, Item, Error> core::Emitter<Item, Error>
    for Box<dyn UnsubscribableEmitter<Item, Error> + 'o>
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

impl<'o, Item, Error> UnsubscribableEmitter<Item, Error>
    for Box<dyn UnsubscribableEmitter<Item, Error> + 'o>
{
    fn is_unsubscribed(&self) -> bool {
        (&**self).is_unsubscribed()
    }
}

impl<Item, Error> core::Emitter<Item, Error>
    for Box<dyn UnsubscribableEmitter<Item, Error> + Send + Sync + 'static>
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

impl<Item, Error> UnsubscribableEmitter<Item, Error>
    for Box<dyn UnsubscribableEmitter<Item, Error> + Send + Sync + 'static>
{
    fn is_unsubscribed(&self) -> bool {
        (&**self).is_unsubscribed()
    }
}
