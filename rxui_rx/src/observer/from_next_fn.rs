use crate::core;
use crate::util;

pub struct NextFnObserver<NextFn> {
    item_consumer: NextFn,
}

impl<NextFn> NextFnObserver<NextFn> {
    pub fn new(item_consumer: NextFn) -> Self {
        Self { item_consumer }
    }
}

impl<NextFn, Cancellable, Item> core::Observer<Cancellable, Item, util::Infallible>
    for NextFnObserver<NextFn>
where
    NextFn: FnMut(Item),
{
    fn on_subscribe(&mut self, _: Cancellable) {}

    fn on_next(&mut self, item: Item) {
        (self.item_consumer)(item)
    }

    fn on_error(&mut self, _: util::Infallible) {
        panic!()
    }

    fn on_completed(&mut self) {}
}

pub fn from_next_fn<NextFn>(item_consumer: NextFn) -> NextFnObserver<NextFn> {
    NextFnObserver::new(item_consumer)
}
