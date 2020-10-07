use crate::util::distribute_value;

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

impl<'o, Cancellable, Item, Error> Observer<Cancellable, Item, Error>
    for Vec<Box<dyn Observer<Cancellable, Item, Error> + 'o>>
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

impl<Cancellable, Item, Error> Observer<Cancellable, Item, Error>
    for Vec<Box<dyn Observer<Cancellable, Item, Error> + Send + 'static>>
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
