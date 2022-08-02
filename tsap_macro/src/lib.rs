extern crate proc_macro;

mod parser;
mod model;

#[cfg(feature="toml")]
mod lower_toml;
#[cfg(feature="toml")]
use lower_toml::*;

mod lower;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn param(args: TokenStream, input: TokenStream) -> TokenStream { 
    let item = parser::parse(args.into(), input.clone().into());
    let model = model::analyze(item);
    let ir = lower::Intermediate::lower(model.clone());
    let tokens = quote!(#ir);

    #[cfg(feature="toml")]
    let res = {
        let ir_toml = Intermediate::lower(model);
        
        quote!(#ir #ir_toml)
    };
    #[cfg(not(feature="toml"))]
    let res = quote!(#ir);

    res.into()
}
