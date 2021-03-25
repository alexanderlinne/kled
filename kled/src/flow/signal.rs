use crate::flow;

#[must_use = "this `Signal` may be an `Err` variant, which should be handled"]
pub enum Signal<Subscription, Item, Error> {
    Subscribe(Subscription),
    Item(Item),
    Error(flow::Error<Error>),
    Completed,
}

impl<Subscription, Item, Error> Signal<Subscription, Item, Error> {
    pub fn is_subscribe(&self) -> bool {
        matches! {self, Self::Subscribe(_)}
    }

    pub fn is_item(&self) -> bool {
        matches! {self, Self::Item(_)}
    }

    pub fn is_error(&self) -> bool {
        matches! {self, Self::Error(_)}
    }

    pub fn is_completed(&self) -> bool {
        matches! {self, Self::Completed}
    }

    pub fn map<UnaryOp, ItemOut>(self, unary_op: UnaryOp) -> Signal<Subscription, ItemOut, Error>
    where
        UnaryOp: FnOnce(Item) -> ItemOut,
    {
        match self {
            Self::Subscribe(subscription) => Signal::Subscribe(subscription),
            Self::Item(item) => Signal::Item(unary_op(item)),
            Self::Error(err) => Signal::Error(err),
            Self::Completed => Signal::Completed,
        }
    }
}
