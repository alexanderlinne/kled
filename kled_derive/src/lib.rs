extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro_error::*;
use syn::{parse_macro_input, AttributeArgs};

mod derive;
use derive::Args;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn operator(args: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);

    let args = match Args::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    derive::derive(&args, item)
}
