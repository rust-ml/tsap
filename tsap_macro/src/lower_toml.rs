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
            #[derive(serde::Serialize, serde::Deserialize)]
            #[serde(tag = "variant")]
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
                    self.0.root[#valname] = toml::Value::try_from(val).unwrap();

                    self
                }

                fn #getter(&self) -> #typ {
                    self.0.root[#valname].clone().try_into().unwrap()
                }
            )
        });

        let impls = quote!(
            impl std::convert::TryFrom<#builder_name> for #name {
                type Error = <#name as ParamGuard>::Error;

                fn try_from(val: #builder_name) -> Result<#name, Self::Error> {
                    let obj: #name = val.0.root().try_into()
                        .map_err(|x| tsap::Error::TomlParse(x))?;

                    obj.check()?;

                    Ok(obj)
                }
            }

            impl #name {
                pub fn from_file<T: AsRef<std::path::Path>>(path: T) -> Result<#builder_name, <#name as ParamGuard>::Error> {
                    tsap::TomlBuilder::from_file(path)
                        .map(|x| #builder_name(x))
                        .map_err(|x| x.into())
                }

                pub fn try_from<V: std::convert::TryInto<tsap::TomlBuilder>>(val: V) -> Result<#builder_name, V::Error> {

                    val.try_into()
                        .map(|x| #builder_name(x))
                        .map_err(|x| x.into())
                }

                pub fn from<V: std::convert::TryInto<tsap::TomlBuilder>>(val: V) -> #builder_name
                    where <V as std::convert::TryInto<tsap::TomlBuilder>>::Error: std::fmt::Debug {
                    Self::try_from(val).unwrap()
                }
            }

            impl #builder_name {
                pub fn amend_file<T: AsRef<std::path::Path>>(mut self, path: T) -> Result<#builder_name, <#name as ParamGuard>::Error> {
                    self.0 = self.0.amend_file(path)?;

                    Ok(self)
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
