pub trait Observer<Cancellable, Item, Error> {
    fn on_subscribe(&mut self, cancellable: Cancellable);
    fn on_next(&mut self, item: Item);
    fn on_error(&mut self, errpr: Error);
    fn on_completed(&mut self);
}

impl<'o, Cancellable, Item, Error> Observer<Cancellable, Item, Error>
    for Box<dyn Observer<Cancellable, Item, Error> + 'o>
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        (&mut **self).on_subscribe(cancellable)
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

impl<Cancellable, Item, Error> Observer<Cancellable, Item, Error>
    for Box<dyn Observer<Cancellable, Item, Error> + Send + 'static>
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        (&mut **self).on_subscribe(cancellable)
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
