use proc_macro2::{TokenStream, Span, Ident};
use quote::{quote, format_ident};
use crate::model::{Model, ModelFields};

#[derive(Debug)]
pub struct Intermediate {
    pub(crate) item: TokenStream,
    pub(crate) impls: TokenStream,
}

impl Intermediate {
    pub(crate) fn lower(model: Model) -> Intermediate {
        let item2 = model.item_definition(Some(false));

        let Model { name, fields, .. } = model;

        let builder_name = format_ident!("{}Builder", name);
        let builder = quote!(
            pub struct #builder_name(tsap::TomlBuilder);
        );

        let item = quote!(
            #builder
        );

        let fields = match fields {
            ModelFields::Struct(fields) => fields.into_iter().map(|(a, b)| (a, Some(b))).collect(),
            ModelFields::Enum(fields) => fields,
        };

        let setter = fields.iter().map(|(name, typ)| { 
            let name = Ident::new(&format!("{}", name).to_lowercase(), Span::call_site());
            let getter = format_ident!("get_{}", name);
            let valname = format!("{}", name);
            let typ = typ.as_ref().unwrap();
            let arg_typ_false = typ.quote(Some(false));

            quote!(
                fn #name<F: FnOnce(#arg_typ_false) -> #arg_typ_false>(mut self, val: F) -> Self {
                    // we can assume a table, because proc-macro only implements on structs
                    let table = self.0.root.as_table_mut().unwrap();
                    let old_val: #arg_typ_false = table.remove(#valname).unwrap().try_into().unwrap();
                    let val: #arg_typ_false = val(old_val);
                    table.insert(#valname.to_string(), toml::Value::try_from(val).unwrap());

                    self
                }

                fn #getter(&self) -> #arg_typ_false {
                    self.0.root[#valname].clone().try_into().unwrap()
                }
            )
        });

        let impls = quote!(
            impl std::convert::TryFrom<#builder_name> for #item2 {
                type Error = <#item2 as ParamGuard>::Error;

                fn try_from(mut val: #builder_name) -> Result<#item2, Self::Error> {
                    val.0.apply()?;

                    let obj: #item2 = val.0.root().try_into()
                        .map_err(|x| tsap::Error::TomlParse(x))?;

                    Ok(obj)
                }
            }

            impl std::convert::From<#item2> for #builder_name {
                fn from(val: #item2) -> #builder_name {
                    let val = toml::Value::try_from(val).unwrap();

                    use std::convert::TryFrom;
                    let val = tsap::TomlBuilder::try_from(val).unwrap();
                    #builder_name(val)
                }
            }

            impl #item2 {
                pub fn from_file<T: AsRef<std::path::Path>>(path: T) -> Result<#builder_name, <#item2 as ParamGuard>::Error> {
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
                pub fn amend_file<T: AsRef<std::path::Path>>(mut self, path: T) -> Result<#builder_name, <#item2 as ParamGuard>::Error> {
                    //self.0 = self.0.amend_file(path)?;

                    Ok(self)
                }

                pub fn amend_args(mut self) -> Result<#builder_name, <#item2 as ParamGuard>::Error> {
                    self.0 = self.0.amend_args()?;

                    Ok(self)
                }
            }

            impl #builder_name {
                #( #setter )*
            }
        );

        Intermediate {
            item,
            impls: impls,
        }
    }
}

impl quote::ToTokens for Intermediate {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Intermediate { item, impls, .. } = &self;

        tokens.extend(quote!(
            #item
            #impls
        ));
    }
}
