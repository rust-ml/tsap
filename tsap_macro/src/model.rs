use syn::{Item, Ident, Type, Fields};
use proc_macro_error::abort;

#[derive(Debug)]
pub struct Model {
    pub(crate) name: Ident,
    pub(crate) item: Item,
    pub(crate) fields: Vec<(Ident, Type)>,
}

pub(crate) fn analyze(item: Item) -> Model {
    let name = match item {
        Item::Enum(ref obj) => obj.ident.clone(),
        Item::Struct(ref obj) => obj.ident.clone(),
        _ => unreachable!()
    };

    let fields = match item {
        Item::Enum(_) => Vec::new(),
        Item::Struct(ref obj) => {
            let fields = match obj.fields.clone() {
                Fields::Named(named) => named.named.into_iter().collect::<Vec<_>>(),
                Fields::Unit => Vec::new(),
                Fields::Unnamed(unnamed) => {
                    abort!(
                        unnamed,
                        "we don't support unnamed structs";
                        help = "`#[params]' can only be used on structs with named fields"
                    )
                }
            };


            fields.into_iter()
                .map(|x| (x.ident.unwrap(), x.ty))
                .collect::<Vec<_>>()
        },
        _ => unreachable!()
    };

    Model {
        name,
        item,
        fields
    }
}
