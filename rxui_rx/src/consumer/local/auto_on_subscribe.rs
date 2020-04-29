use crate::core;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct AutoOnSubscribe<Observer, Item, Error> {
    observer: Observer,
    observed: Rc<RefCell<bool>>,
    phantom: PhantomData<(Item, Error)>,
}

impl<'o, Observer, Item, Error> AutoOnSubscribe<Observer, Item, Error>
where
    Observer: core::Observer<core::LocalSubscription, Item, Error> + 'o,
{
    pub fn new(mut observer: Observer) -> Self {
        let observed = Rc::new(RefCell::new(true));
        observer.on_subscribe(core::LocalSubscription::new(observed.clone()));
        Self {
            observer,
            observed,
            phantom: PhantomData,
        }
    }
}

impl<'o, Observer, Item, Error> core::Consumer<Item, Error>
    for AutoOnSubscribe<Observer, Item, Error>
where
    Observer: core::Observer<core::LocalSubscription, Item, Error> + 'o,
{
    fn on_next(&mut self, item: Item) {
        self.observer.on_next(item);
    }

    fn on_error(&mut self, error: Error) {
        self.observer.on_error(error);
    }

    fn on_completed(&mut self) {
        self.observer.on_completed();
    }
}

impl<'o, Observer, Item, Error> core::CancellableConsumer<Item, Error>
    for AutoOnSubscribe<Observer, Item, Error>
where
    Observer: core::Observer<core::LocalSubscription, Item, Error> + 'o,
{
    fn is_cancelled(&self) -> bool {
        !*self.observed.borrow()
    }
}

#[cfg(test)]
mod tests {
    use crate::observer;
    use crate::prelude::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn unsubscribe() {
        let vec = vec![0, 1, 2, 3];
        let sum = Rc::new(RefCell::new(0));
        let sum_move = sum.clone();
        vec.into_observable().subscribe(observer::from_fn(
            |sub: LocalSubscription| {
                sub.cancel();
            },
            move |v| *sum_move.borrow_mut() += v,
            |_| {},
            || {},
        ));
        assert_eq!(*sum.borrow(), 0);
    }
}