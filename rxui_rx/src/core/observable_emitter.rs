use crate::util::distribute_value;

pub trait ObservableEmitter<Item, Error> {
    fn on_next(&mut self, item: Item);
    fn on_error(&mut self, error: Error);
    fn on_completed(&mut self);

    fn is_cancelled(&self) -> bool {
        false
    }
}

impl<'o, Item, Error> ObservableEmitter<Item, Error>
    for Box<dyn ObservableEmitter<Item, Error> + 'o>
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

    fn is_cancelled(&self) -> bool {
        (&**self).is_cancelled()
    }
}

impl<Item, Error> ObservableEmitter<Item, Error>
    for Box<dyn ObservableEmitter<Item, Error> + Send + 'static>
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

    fn is_cancelled(&self) -> bool {
        (&**self).is_cancelled()
    }
}

impl<'o, Item, Error> ObservableEmitter<Item, Error>
    for Vec<Box<dyn ObservableEmitter<Item, Error> + 'o>>
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

    fn is_cancelled(&self) -> bool {
        self.iter().fold(true, |is_cancelled, item| {
            is_cancelled && item.is_cancelled()
        })
    }
}

impl<Item, Error> ObservableEmitter<Item, Error>
    for Vec<Box<dyn ObservableEmitter<Item, Error> + Send + 'static>>
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

    fn is_cancelled(&self) -> bool {
        self.iter().fold(true, |is_cancelled, item| {
            is_cancelled && item.is_cancelled()
        })
    }
}
