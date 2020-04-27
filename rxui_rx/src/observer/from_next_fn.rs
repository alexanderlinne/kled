use crate::core;

pub struct NextFnObserver<NextFn> {
    item_consumer: NextFn,
}

impl<NextFn> NextFnObserver<NextFn> {
    pub fn new(item_consumer: NextFn) -> Self {
        Self { item_consumer }
    }
}

impl<NextFn, Subscription, Item> core::Observer<Subscription, Item, ()> for NextFnObserver<NextFn>
where
    NextFn: FnMut(Item),
{
    fn on_subscribe(&mut self, _: Subscription) {}

    fn on_next(&mut self, item: Item) {
        (self.item_consumer)(item)
    }

    fn on_error(&mut self, _: ()) {
        panic!()
    }

    fn on_completed(&mut self) {}
}

pub fn from_next_fn<NextFn>(item_consumer: NextFn) -> NextFnObserver<NextFn> {
    NextFnObserver::new(item_consumer)
}
