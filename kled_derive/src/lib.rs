extern crate inflector;
extern crate proc_macro;
extern crate quote;
extern crate syn;

mod reactive_operator;

use proc_macro::TokenStream;

#[proc_macro_derive(reactive_operator, attributes(upstream, reactive_operator))]
pub fn derive_reactive_operator(item: TokenStream) -> TokenStream {
    reactive_operator::derive(item)
}
