#[must_use = "this `Signal` may be an `Err` variant, which should be handled"]
pub enum Signal<Subscription, Item, Err> {
    Subscribe(Subscription),
    Item(Item),
    Err(Err),
    Completed,
}

impl<Subscription, Item, Err> Signal<Subscription, Item, Err> {
    pub fn is_subscribe(&self) -> bool {
        matches! {self, Self::Subscribe(_)}
    }

    pub fn is_item(&self) -> bool {
        matches! {self, Self::Item(_)}
    }

    pub fn is_err(&self) -> bool {
        matches! {self, Self::Err(_)}
    }

    pub fn is_completed(&self) -> bool {
        matches! {self, Self::Completed}
    }

    pub fn map<UnaryOp, ItemOut>(self, unary_op: UnaryOp) -> Signal<Subscription, ItemOut, Err>
    where
        UnaryOp: FnOnce(Item) -> ItemOut,
    {
        match self {
            Self::Subscribe(subscription) => Signal::Subscribe(subscription),
            Self::Item(item) => Signal::Item(unary_op(item)),
            Self::Err(err) => Signal::Err(err),
            Self::Completed => Signal::Completed,
        }
    }
}
