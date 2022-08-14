use proc_macro2::TokenStream;
use quote::{quote, format_ident};

use crate::model::Model;

#[derive(Debug)]
pub struct Intermediate {
    pub(crate) item: TokenStream,
    pub(crate) builder: TokenStream,
    pub(crate) impls: TokenStream,
}

impl Intermediate {
    pub(crate) fn lower(model: Model) -> Intermediate {
        let Model { name, item, fields } = model;

        let item = quote!(
            use serde::{Serialize, Deserialize};

            #[derive(Serialize, Deserialize)]
            #item
        );

        let builder_name = format_ident!("{}Builder", name);
        let builder = quote!(
            pub struct #builder_name(tsap::TomlBuilder);
        );
        let setter = fields.iter().map(|(name, typ)| { 
            let getter = format_ident!("get_{}", name);
            let valname = format!("{}", name);

            quote!(
                fn #name(mut self, val: #typ) -> Self {
                    self.0.root[#valname] = Value::try_from(val).unwrap();

                    self
                }

                fn #getter(&self) -> #typ {
                    self.0.root[#valname].clone().try_into().unwrap()
                }
            )
        });

        let impls = quote!(
            use std::convert::TryFrom;

            impl TryFrom<#builder_name> for #name {
                type Error = <#name as ParamGuard>::Error;

                fn try_from(val: #builder_name) -> Result<#name, Self::Error> {
                    let obj: #name = val.0.root().try_into()
                        .map_err(|x| tsap::Error::TomlParse(x))?;

                    obj.check()?;

                    Ok(obj)
                }
            }

            use std::path::Path;
            use toml::Value;
            use tsap::TomlBuilder;

            impl #name {
                pub fn from_file<T: AsRef<Path>>(path: T) -> Result<#builder_name, <#name as ParamGuard>::Error> {
                    TomlBuilder::from_file(path)
                        .map(|x| #builder_name(x))
                        .map_err(|x| x.into())
                }
            }

            impl #builder_name {
                #( #setter )*
            }
        );

        Intermediate {
            item,
            builder: builder,
            impls: impls,
        }
    }
}
