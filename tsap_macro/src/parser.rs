use syn::{Expr, Item, ItemStruct};
use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site};

type Ast = ItemStruct;

pub(crate) fn parse(args: TokenStream, input: TokenStream) -> Ast {
    const ERROR: &str = "this attribute takes no arguments";
    const HELP: &str = "use #[param]";

    if !args.is_empty() {
        if let Ok(expr) = syn::parse2::<Expr>(args) {
            abort!(expr, ERROR; help = HELP)
        } else {
            abort_call_site!(ERROR; help = HELP)
        }
    }

    let parsed = match syn::parse2::<Item>(input) {
        Ok(Item::Struct(item)) => item,
        Ok(item) => {
            abort!(
                item,
                "item is not a struct";
                help = "`#[params]` can only be used on structs"
            )
        }
        Err(_) => unreachable!(), // ?
    };

    if !parsed.generics.params.is_empty() {
        abort!(
            parsed,
            "we don't support generics";
            help = "`#[params]' can only be used on structs without generics"
        )
    }

    parsed
}

#[cfg(test)]
mod tests {
    use super::parse;
    use quote::quote;

    #[test]
    fn valid_struct() {
        parse(
            quote!(),
            quote!(
                #[param]
                struct Param {
                    ntrees: usize,
                    max_depth: usize
                }
            )
        );
    }

    #[test]
    #[should_panic]
    fn invalid_generics() {
        parse(
            quote!(),
            quote!(
                #[param]
                struct Param<T> {
                    ntrees: usize,
                    max_depth: T
                }
            )
        );
    }

    #[test]
    #[should_panic]
    fn invalid_enum() {
        parse(
            quote!(),
            quote!(
                #[param]
                enum Param {
                    ntrees: usize,
                    max_depth: usize
                }
            )
        );
    }
}
