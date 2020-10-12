use crate::flow;

#[blanket(derive(Mut, Box))]
pub trait Subscriber<Subscription, Item, Error> {
    fn on_subscribe(&mut self, subscription: Subscription);
    fn on_next(&mut self, item: Item);
    fn on_error(&mut self, error: flow::Error<Error>);
    fn on_completed(&mut self);
}
