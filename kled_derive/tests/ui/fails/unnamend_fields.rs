#[macro_use]
extern crate kled_derive;

#[operator(type = "flow")]
pub struct Map<UnaryOp>(UnaryOp);

fn main() {}
