use syn::{Item, Ident, Type, Fields};
use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use proc_macro_error::abort;

#[derive(Debug)]
pub struct Model {
    name: Ident,
    item: Item,
    fields: Vec<(Ident, Type)>,
}

impl Model {
    pub(crate) fn builder(item: &Item) -> Option<Item> {
        let mut struc = match item {
            Item::Struct(obj) => obj.clone(),
            _ => return None
        };

        struc.ident = format_ident!("{}Builder", struc.ident);

        // prefix each field
        for field in &mut struc.fields {
            field.ident = Some(format_ident!("val_{}", field.ident.as_ref().unwrap()));
        }

        Some(Item::Struct(struc))
    }

    pub(crate) fn to_tokens(self) -> TokenStream {
        let Model { name, item, fields } = self;

        let builder = Model::builder(&item);
        let name = format_ident!("{}Builder", name);
        let impls = fields.into_iter()
            .map(|(name, typ)| {
                let valname = format_ident!("val_{}", name);
                let getter = format_ident!("get_{}", name);

                quote!(
                    fn #name(mut self, val: #typ) -> Self {
                        self.#valname = val;

                        self
                    }

                    fn #getter(&self) -> &#typ {
                        &self.#valname
                    }
                )
            });


        quote!(
            #item

            #builder

            impl #name {
                #( 
                    #impls
                )*
            }
        )
    }
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
