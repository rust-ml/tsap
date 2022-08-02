use syn::{Expr, Item, GenericParam, Fields};
use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site};

type Ast = Item;

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

    let (parsed, first_generic) = match syn::parse2::<Item>(input) {
        Ok(Item::Struct(item)) => {
            let first = item.generics.params.first().map(|x| x.clone());

            (
                Item::Struct(item),
                first
            )
        },
        Ok(Item::Enum(item)) => {
            for variant in &item.variants {
                match variant.fields {
                    Fields::Named(_) => {
                        abort!(
                            variant,
                            "named enum variants are not supported";
                            help = "use unnamed variant"
                        )
                    },
                    _ => {}
                }
            }

            let first = item.generics.params.first().map(|x| x.clone());

            (
                Item::Enum(item),
                first
            )
        },
        Ok(item) => {
            abort!(
                item,
                "item is not a struct or enum";
                help = "`#[params]` can only be used on structs or enums"
            )
        },
        Err(err) => {
            panic!("could not parse item: {}", err);
        }
    };


    match first_generic {
        None => {
            abort!(
                parsed,
                "parameter sets should have a const boolean to indicate checking";
                help = "add a const boolean as your first generic"
            )
        },
        Some(GenericParam::Type(_)) | Some(GenericParam::Lifetime(_)) => {
            abort!(
                parsed,
                "parameter sets should have a const boolean to indicate checking";
                help = "add a const boolean as your first generic"
            )
        },
        _ => {}
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
                struct Param<const C: bool, T> {
                    ntrees: usize,
                    max_depth: usize
                }
            )
        );
    }

    #[test]
    fn valid_enum() {
        parse(
            quote!(),
            quote!(
                #[param]
                enum Param<const C: bool, T> {
                    SVClassifier,
                    IsolationForest(T),
                }
            )
        );
    }

}
