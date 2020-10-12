#[blanket(derive(Mut, Box))]
pub trait Observer<Cancellable, Item, Error> {
    fn on_subscribe(&mut self, cancellable: Cancellable);
    fn on_next(&mut self, item: Item);
    fn on_error(&mut self, errpr: Error);
    fn on_completed(&mut self);
}
