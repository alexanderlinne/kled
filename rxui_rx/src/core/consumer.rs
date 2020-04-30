pub trait Consumer<Item, Error> {
    fn on_next(&mut self, item: Item);
    fn on_error(&mut self, errpr: Error);
    fn on_completed(&mut self);
}

impl<'o, Item, Error> Consumer<Item, Error> for Box<dyn Consumer<Item, Error> + 'o> {
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

impl<Item, Error> Consumer<Item, Error> for Box<dyn Consumer<Item, Error> + Send + 'static> {
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
