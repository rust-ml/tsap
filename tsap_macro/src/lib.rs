extern crate proc_macro;

mod parser;
mod model;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn param(args: TokenStream, input: TokenStream) -> TokenStream { 
    let item = parser::parse(args.into(), input.clone().into());
    let model = model::analyze(item);
    let tokens = model.to_tokens();
    println!("{}", &tokens);
    tokens.into()
}
