pub trait Observer<Subscription, Item, Error> {
    fn on_subscribe(&mut self, subscription: Subscription);
    fn on_next(&mut self, item: Item);
    fn on_error(&mut self, errpr: Error);
    fn on_completed(&mut self);
}

impl<'o, Subscription, Item, Error> Observer<Subscription, Item, Error>
    for Box<dyn Observer<Subscription, Item, Error> + 'o>
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        (&mut **self).on_subscribe(subscription)
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

impl<Subscription, Item, Error> Observer<Subscription, Item, Error>
    for Box<dyn Observer<Subscription, Item, Error> + Send + Sync + 'static>
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        (&mut **self).on_subscribe(subscription)
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

pub trait AutoUnsubscribeObserver<Subscription, Item, Error>:
    Observer<Subscription, Item, Error>
{
    fn is_unsubscribed(&self) -> bool;
}
