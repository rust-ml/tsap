extern crate proc_macro;

mod parser;
mod model;

#[cfg(feature="toml")]
mod lower_toml;
#[cfg(feature="toml")]
use lower_toml::*;

#[cfg(not(feature="toml"))]
mod lower;
#[cfg(not(feature="toml"))]
use lower::*;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn param(args: TokenStream, input: TokenStream) -> TokenStream { 
    let item = parser::parse(args.into(), input.clone().into());
    let model = model::analyze(item);
    let ir = Intermediate::lower(model);
    let tokens = codegen(ir);
    println!("{}", &tokens);
    tokens.into()
}

fn codegen(ir: Intermediate) -> TokenStream {
    let Intermediate {item, builder, impls} = ir;

    quote!(
        #item
        #builder
        #impls
    ).into()
}
