use syn::{Item, Ident, Type, Fields};
use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use proc_macro_error::abort;

use crate::model::Model;

#[derive(Debug)]
pub struct Intermediate {
    pub(crate) item: Item,
    pub(crate) builder: Option<TokenStream>,
    pub(crate) impls: TokenStream,
}

impl Intermediate {
    pub(crate) fn lower(model: Model) -> Intermediate {
        let Model { name, item, fields } = model;

        if let Item::Enum(_) = &item {
            return Intermediate {
                item,
                builder: None,
                impls: quote!()
            };
        }

        let builder_name = format_ident!("{}Builder", name);
        let builder = quote!(
            pub struct #builder_name {
                inner: #name
            }
        );

        let impls = fields.iter()
            .map(|(name, typ)| {
                let getter = format_ident!("get_{}", name);

                quote!(
                    fn #name(mut self, val: #typ) -> #builder_name {
                        self.#name = val;

                        self.into()
                    }

                    fn #getter(&self) -> &#typ {
                        &self.#name
                    }
                )
            });

        let builder_impls = fields.iter()
            .map(|(name, typ)| {
                let getter = format_ident!("get_{}", name);

                quote!(
                    fn #name(mut self, val: #typ) -> Self {
                        self.inner.#name = val;

                        self
                    }

                    fn #getter(&self) -> &#typ {
                        &self.inner.#name
                    }
                )
            });

        let mut impls = quote!(
            impl #name {
                #(
                    #impls
                )*
            }

            impl #builder_name {
                #( 
                    #builder_impls
                )*

                pub fn build(self) -> Result<#name, <#name as ParamGuard>::Error> {
                    std::convert::TryFrom::try_from(self)
                }
            }

            impl std::convert::TryFrom<#builder_name> for #name {
                type Error = <#name as ParamGuard>::Error;

                fn try_from(val: #builder_name) -> Result<#name, Self::Error> {
                    val.inner.check()?;

                    Ok(val.inner)
                }
            }

            impl std::convert::From<#name> for #builder_name {
                fn from(val: #name) -> #builder_name {
                    #builder_name {
                        inner: val,
                    }
                }
            }
        );

        if cfg!(feature = "impl_try") {
            impls = quote!(
                #impls

                impl std::ops::Try for #builder_name {
                    type Output = #name;
                    type Residual = Result<std::convert::Infallible, <#name as ParamGuard>::Error>;

                    fn from_output(output: Self::Output) -> Self {
                        #builder_name {
                            inner: output
                        }
                    }

                    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
                        match self.build() {
                            Ok(v) => std::ops::ControlFlow::Continue(v),
                            Err(e) => std::ops::ControlFlow::Break(Err(e)),
                        }
                    }
                }

                impl std::ops::FromResidual<<#builder_name as std::ops::Try>::Residual> for #builder_name {
                    #[track_caller]
                    fn from_residual(residual: Result<std::convert::Infallible, <#name as ParamGuard>::Error>) -> Self {
                        #builder_name {
                            inner: #name::default()
                        }
                    }
                }
            );
        }

        Intermediate {
            item,
            builder: Some(builder),
            impls,
        }
    }
}

