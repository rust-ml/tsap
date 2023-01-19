use syn::{Item, Ident};
use proc_macro2::TokenStream;
use quote::{quote, format_ident};

use crate::model::{Model, ModelFields, ModelType};

#[derive(Debug)]
pub struct Intermediate {
    pub(crate) item: Item,
    pub(crate) impls: TokenStream,
}

impl Intermediate {
    fn lower_struct(model: &Model, fields: &Vec<(Ident, ModelType)>) -> TokenStream {
        let Model { name, check_name, rem_param_types, .. } = model;

        let composer = fields.iter()
            .map(|(field_name, typ)| {
                let getter = format_ident!("get_{}", field_name);
                let item = model.item_definition(Some(false));
                let arg_typ_false = typ.quote(Some(false));
                let arg_typ = typ.quote(None);

                //dbg!(&getter, &arg_typ_false, &arg_typ);

                let setter = if typ.has_const_name() {
                    quote!(
                        tmp.#field_name = val(tmp.#field_name).unseal();
                    )
                } else {
                    quote!(
                        tmp.#field_name = val(tmp.#field_name);
                    )
                };

                quote!(
                    pub fn #field_name<F: FnOnce(#arg_typ_false) -> #arg_typ_false>(mut self, val: F) -> #item {
                        let mut tmp = self.unseal();
                        #setter

                        tmp

                    }

                    pub fn #getter(&self) -> &#arg_typ {
                        &self.#field_name
                    }
                )
            });

        // generate fields of the unseal function
        let unseal_fields = fields.iter()
            .map(|(field_name, typ)| {
                if typ.has_const_name() {
                    quote!(#field_name: self.#field_name.unseal())
                } else {
                    quote!(#field_name: self.#field_name)
                }
            }).collect::<Vec<_>>();

        // generate fields of the try_into function
        let from_fields = fields.iter()
            .map(|(field_name, typ)| {
                if typ.has_const_name() {
                    quote!(#field_name: val.#field_name.try_into()?)
                } else {
                    quote!(#field_name: val.#field_name)
                }
            });

        quote!(
            impl<const #check_name: bool, #( #rem_param_types)*> #name<C, #( #rem_param_types)*> {
                #(#composer)*

                pub fn unseal(self) -> #name<false, #( #rem_param_types)*> {
                    #name {
                        #(#unseal_fields,)*
                    }
                }
            }

            impl<#( #rem_param_types)*> std::convert::TryFrom<#name<false, #( #rem_param_types)*>> for #name<true, #( #rem_param_types)*> {
                type Error = <#name<false, #( #rem_param_types)*> as ParamGuard>::Error;

                fn try_from(val: #name<false, #( #rem_param_types)*>) -> Result<Self, Self::Error> {
                    // check self
                    val.check()?;

                    // reconstruct self with new const generic
                    Ok(
                        #name {
                            #(#from_fields,)*
                        }
                    )
                }
            }
        )
    }

    fn lower_enum(model: &Model, variants: &Vec<(Ident, Option<ModelType>)>) -> TokenStream {
        let Model { name, check_name, rem_param_types, .. } = model;

        let composer = variants.iter()
            .map(|(variant_name, typ)| {
                let mapper = format_ident!("{}", format!("{}", variant_name).to_lowercase());
                let item = model.item_definition(Some(false));

                if let Some(typ) = typ {
                    let arg_typ_false = typ.quote(Some(false));

                    let setter = if typ.has_const_name() {
                        quote!(
                            tmp = val(tmp).unseal();
                        )
                    } else {
                        quote!(
                            tmp = val(tmp);
                        )
                    };

                    quote!(
                        pub fn #mapper<F: FnOnce(#arg_typ_false) -> #arg_typ_false>(mut self, val: F) -> #item {
                            let mut tmp = match self {
                                Self::#variant_name(x) => x,
                                _ => #arg_typ_false::default(),
                            };

                            #setter

                            #name::#variant_name(tmp)
                        }
                    )
                } else {
                    quote!(
                        fn #mapper(self) -> #name<false, #( #rem_param_types )*> {
                            #name::#variant_name
                        }
                    )
                }
            });

        let decompose = variants.iter()
            .map(|(variant_name, typ)| {

                if let Some(typ) = typ {
                    if typ.has_const_name() {
                        quote!(
                            #name::#variant_name(val) => #name::#variant_name(val.try_into()?)
                        )
                    } else {
                        quote!(
                            #name::#variant_name(val) => #name::#variant_name(val)
                        )
                    }
                } else {
                    quote!(#name::#variant_name => #name::#variant_name)
                }
            })
            .collect::<Vec<_>>();

        let checking = variants.iter()
            .filter_map(|(variant_name, typ)| {
                if typ.as_ref().map(|x| x.has_const_name()).unwrap_or(false) {
                    Some(quote!(self.#variant_name.check()?))
                } else {
                    None
                }
            });
        
        // generate fields of the unseal function
        let unseal_fields = variants.iter()
            .map(|(variant_name, typ)| {
                if let Some(typ) = typ {
                    if typ.has_const_name() {
                        quote!(Self::#variant_name(x) => #name::#variant_name(x.unseal()))
                    } else {
                        quote!(Self::#variant_name(x) => #name::#variant_name(x))
                    }
                } else {
                    quote!(Self::#variant_name => #name::#variant_name)
                }
            }).collect::<Vec<_>>();

        
        let item = model.item_definition(None);
        quote!(
            impl<const #check_name: bool, #( #rem_param_types)*> #name<C, #( #rem_param_types)*> {
                #(
                    #composer
                )*

                pub fn unseal(self) -> #name<false, #( #rem_param_types)*> {
                    match self {
                        #(#unseal_fields,)*
                    }
                }
            }

            impl<const #check_name: bool, #( #rem_param_types)*> ParamGuard for #item {
                type Error = tsap::Error;

                fn check(&self) -> Result<(), Self::Error> {
                    #(#checking)*

                    Ok(())
                }
            }

            impl<#( #rem_param_types)*> std::convert::TryFrom<#name<false, #( #rem_param_types)*>> for #name<true, #( #rem_param_types)*> {
                type Error = <#name<false, #( #rem_param_types)*> as ParamGuard>::Error;

                fn try_from(val: #name<false, #( #rem_param_types)*>) -> Result<Self, Self::Error> {
                    val.check()?;

                    use std::convert::TryInto;
                    Ok(
                        match val {
                        #(
                            #decompose,
                        )*
                        }
                    )
                }
            }
        )
    }

    pub(crate) fn lower(model: Model) -> Intermediate {
        let impls = match &model.fields {
            ModelFields::Struct(fields) => Self::lower_struct(&model, fields),
            ModelFields::Enum(fields) => Self::lower_enum(&model, fields),
        };

        let Model { name, rem_param_types, .. } = model;

        let mut impls = quote!(
            #impls

            impl< #( #rem_param_types)*> #name<false, #( #rem_param_types)*> {
                fn build(self) -> Result<#name<true, #( #rem_param_types)*>, <#name<false, #( #rem_param_types)*> as ParamGuard>::Error> {
                    use std::convert::TryInto;

                    self.try_into()
                }
            }
        );

        if cfg!(feature = "impl_try") {
            impls = quote!(
                #impls

                impl< #( #rem_param_types)*> std::ops::Try for #name<false,  #( #rem_param_types)*> {
                    type Output = #name<true, #( #rem_param_types)*>;
                    type Residual = Result<std::convert::Infallible, <#name<false, #( #rem_param_types)*> as ParamGuard>::Error>;

                    fn from_output(output: Self::Output) -> Self {
                        output.unseal()
                    }

                    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
                        match self.build() {
                            Ok(v) => std::ops::ControlFlow::Continue(v),
                            Err(e) => std::ops::ControlFlow::Break(Err(e)),
                        }
                    }
                }

                impl< #( #rem_param_types)*> std::ops::FromResidual<<#name<false,  #( #rem_param_types)*> as std::ops::Try>::Residual> for #name<false,  #( #rem_param_types)*> {
                    #[track_caller]
                    fn from_residual(residual: Result<std::convert::Infallible, <Self as ParamGuard>::Error>) -> Self {
                        <#name<true, #( #rem_param_types)*>>::default().unseal()
                    }
                }
            );
        }

        Intermediate {
            item: model.item,
            impls
        }
    }
}

