#[macro_use]
extern crate kled_derive;

#[operator(type = "flow", subscription = "struct Foo {}")]
pub struct Map<ItemOut, UnaryOp> {}

fn main() {}
