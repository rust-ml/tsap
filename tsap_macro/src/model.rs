use syn::{Item, Ident, Type, Fields, GenericParam, FieldsUnnamed, PathArguments, AngleBracketedGenericArguments, GenericArgument, TypePath, punctuated::Punctuated, token::{Comma, Colon2}, PathSegment};
use proc_macro_error::abort;
use quote::quote;
use proc_macro2::{TokenStream, Span};

#[derive(Debug)]
pub struct ModelType {
    name: Ident,
    typs: Punctuated<PathSegment, Colon2>,
    const_name: Option<Ident>,
}

impl ModelType {
    pub fn new(typ: &Type, const_name: &Ident) -> ModelType {
        //dbg!(&typ);
        let first_seg = match typ {
            Type::Path(TypePath { ref path, .. }) => path.segments.first().unwrap(),
            _ => panic!("Field not a type path!"),
        };
        let name = first_seg.ident.clone();

        let typs = if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { ref args, ..}) = first_seg.arguments {
            match args.first() {
                Some(GenericArgument::Type(Type::Path(TypePath { path, .. }))) => path.segments.clone(),
                _ => panic!("bla"),
            }
        } else {
            Punctuated::new()
        };

        if !typs.is_empty() && &typs[0].ident == const_name {
            ModelType {
                name,
                typs,
                const_name: Some(const_name.clone()),
            }
        } else {
            ModelType {
                name,
                typs,
                const_name: None,
            }
        }
    }

    pub fn quote(&self, checked: Option<bool>) -> TokenStream {
        let name = &self.name;
        if self.typs.is_empty() {
            return quote!(#name);
        }

        let mut typs = self.typs.clone();

        if let Some(const_name) = &self.const_name {
            let check_param = match checked {
                Some(true) => Ident::new("true", Span::call_site()),
                Some(false) => Ident::new("false", Span::call_site()),
                None => const_name.clone(),
            };

            typs.first_mut().map(|x| *x = PathSegment { ident: check_param, arguments: PathArguments::None });
        } 

        quote!(#name<#typs>)
    }

    pub fn has_const_name(&self) -> bool {
        self.const_name.is_some()
    }
}

#[derive(Debug)]
pub enum ModelFields {
    Enum(Vec<(Ident, Option<ModelType>)>),
    Struct(Vec<(Ident, ModelType)>),
}

#[derive(Debug)]
pub struct Model {
    pub(crate) name: Ident,
    pub(crate) item: Item,
    pub(crate) fields: ModelFields,
    pub(crate) check_name: Ident,
    pub(crate) rem_param_types: Vec<GenericParam>,
}

impl Model {
    pub(crate) fn param_args(&self) -> Punctuated<Ident, Comma> {
        self.rem_param_types.iter()
            .map(|x| match x {
                GenericParam::Type(t) => t.ident.clone(),
                GenericParam::Lifetime(l) => l.lifetime.ident.clone(),
                GenericParam::Const(c) => c.ident.clone(),
            })
            .collect()
    }

    pub(crate) fn item_definition(&self, checked: Option<bool>) -> TokenStream {
        let check_param = match checked {
            Some(true) => Ident::new("true", Span::call_site()),
            Some(false) => Ident::new("false", Span::call_site()),
            None => self.check_name.clone(),
        };

        let (name, args) = (&self.name, self.param_args());
        quote!(#name<#check_param, #args>)
    }
}

pub(crate) fn analyze(item: Item) -> Model {
    let name = match item {
        Item::Enum(ref obj) => obj.ident.clone(),
        Item::Struct(ref obj) => obj.ident.clone(),
        _ => unreachable!()
    };

    let mut param_types = match item {
        Item::Enum(ref obj) => obj.generics.params.clone(),
        Item::Struct(ref obj) => obj.generics.params.clone(),
        _ => unreachable!(),
    }.into_iter();

    let check_name = match param_types.next() {
        Some(GenericParam::Const(const_param)) => const_param.ident,
        _ => unreachable!()
    };
    let rem_param_types = param_types.collect();

    let fields = match item {
        Item::Enum(ref obj) => {
            let res = obj.variants.iter()
                .map(|x| {
                    let inner = match &x.fields {
                        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                            if unnamed.len() > 1 {
                                abort!(
                                    obj,
                                    "we don't support variants with more than one field";
                                    help = "`#[params]' can only be used on enum with single field variants"
                                )
                            }

                            unnamed.first()
                                .map(|x| ModelType::new(&x.ty, &check_name))
                        },
                        Fields::Unit => None,
                        _ => unreachable!(),
                    };

                    (x.ident.clone(), inner)
                }).collect::<Vec<_>>();

            ModelFields::Enum(res)

        },
        Item::Struct(ref obj) => {
            let fields = match obj.fields.clone() {
                Fields::Named(named) => {
                    named.named.into_iter()
                        .map(|x| (x.ident.clone().unwrap(), ModelType::new(&x.ty, &check_name)))
                        .collect::<Vec<_>>()
                },
                Fields::Unit => Vec::new(),
                Fields::Unnamed(unnamed) => {
                    abort!(
                        unnamed,
                        "we don't support unnamed structs";
                        help = "`#[params]' can only be used on structs with named fields"
                    )
                }
            };


            ModelFields::Struct(fields)
        },
        _ => unreachable!()
    };

    Model {
        name,
        item,
        fields,
        check_name,
        rem_param_types,
    }
}
