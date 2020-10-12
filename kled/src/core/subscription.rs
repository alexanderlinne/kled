#[blanket(derive(Ref, Box, Rc))]
pub trait Subscription {
    fn cancel(&self);

    fn is_cancelled(&self) -> bool;

    fn request(&self, count: usize);
}
