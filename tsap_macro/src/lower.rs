use syn::{Item, Ident, Type, Fields};
use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use proc_macro_error::abort;

use crate::model::Model;

#[derive(Debug)]
pub struct Intermediate {
    pub(crate) item: Item,
    pub(crate) builder: Option<Item>,
    pub(crate) impls: TokenStream,
}

impl Intermediate {
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

    pub(crate) fn lower(model: Model) -> Intermediate {
        let Model { name, item, fields } = model;

        let builder = Intermediate::builder(&item);
        let builder_name = format_ident!("{}Builder", name);
        let impls = fields.iter()
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

        let assigns = fields.iter()
            .map(|(name, typ)| {
                let valname = format_ident!("val_{}", name);

                quote!(
                    #name: val.#valname
                )
            });


        let impls = quote!(
            use std::convert::TryFrom;
            impl #builder_name {
                #( 
                    #impls
                )*
            }

            impl TryFrom<#builder_name> for #name {
                type Error = <#name as ParamGuard>::Error;

                fn try_from(val: #builder_name) -> Result<#name, Self::Error> {
                    let res = #name {
                        #( 
                            #assigns 
                        )*
                    };

                    res.check()?;

                    Ok(res)
                }
            }

            //impl From<#builder_name> for #name {
            //    fn from(val: #builder_name) -> #name {
            //        Self::try_from(val).unwrap()
            //    }
            //}
        );

        Intermediate {
            item,
            builder,
            impls,
        }
    }
}

