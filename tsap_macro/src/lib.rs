extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{proc_macro_error, abort, abort_call_site};
use quote::quote;

use syn::{Expr, Item, ItemStruct};

type Ast = ItemStruct;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn param(args: TokenStream, input: TokenStream) -> TokenStream { 
    let item = parse(args.into(), input.clone().into());
    dbg!(item);
    input
}

fn parse(args: TokenStream2, input: TokenStream2) -> Ast {
    const ERROR: &str = "this attribute takes no arguments";
    const HELP: &str = "use #[param]";

    if !args.is_empty() {
        if let Ok(expr) = syn::parse2::<Expr>(args) {
            abort!(expr, ERROR; help = HELP)
        } else {
            abort_call_site!(ERROR; help = HELP)
        }
    }

    match syn::parse2::<Item>(input) {
        Ok(Item::Struct(item)) => item,
        Ok(item) => {
            abort!(
                item,
                "item is not a struct";
                help = "`#[params]` can only be used on structs"
            )
        }
        Err(_) => unreachable!(), // ?
    }
}
