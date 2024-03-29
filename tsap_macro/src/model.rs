use syn::{Item, Ident, Type, Fields, GenericParam, FieldsUnnamed, punctuated::Punctuated, token::Comma, PathSegment, visit::{self, Visit}, visit_mut::{self, VisitMut}, Visibility};
use proc_macro_error::abort;
use quote::quote;
use proc_macro2::{TokenStream, Span};

/// Find the const generic segment in a type
struct FindCheck<'a> {
    found: bool,
    const_name: &'a Ident,
}

impl<'ast> Visit<'ast> for FindCheck<'ast> {
    fn visit_path_segment(&mut self, node: &'ast PathSegment) {
        self.found |= &node.ident == self.const_name;

        visit::visit_path_segment(self, node);
    }
}

/// Replace the const generic segment in a type
struct ReplaceCheck {
    find: Ident,
    replace: Ident,
}

impl VisitMut for ReplaceCheck {
    fn visit_path_segment_mut(&mut self, node: &mut PathSegment) {
        if &node.ident == &self.find {
            node.ident = self.replace.clone();
        }

        visit_mut::visit_path_segment_mut(self, node);
    }
}

/// Type of a field, either named in struct or unnamed in enums
///
/// Carries information whether we want to track check status of wrapped type
/// in `const_name` optionality.
#[derive(Debug, Clone)]
pub struct ModelType {
    wrapped: Type,
    const_name: Option<Ident>,
}

impl ModelType {
    /// Construct a field type with const generic parameter
    ///
    /// Checks if const generic parameter exists in the wrapped type
    pub fn new(typ: &Type, const_name: &Ident) -> ModelType {
        let mut visitor = FindCheck { found: false, const_name };
        visitor.visit_type(typ);
        
        let const_name = match visitor.found {
            true => Some(const_name.clone()),
            false => None,
        };

        ModelType {
            wrapped: typ.clone(),
            const_name,
        }
    }

    /// Create a token stream from the type and fills in definitions of const
    /// generic parameter
    pub fn quote(&self, checked: Option<bool>) -> TokenStream {
        if let Some(const_name) = &self.const_name {
            let check_param = match checked {
                Some(true) => Ident::new("true", Span::call_site()),
                Some(false) => Ident::new("false", Span::call_site()),
                None => const_name.clone(),
            };

            let mut ret_type = self.wrapped.clone();
            let mut replace = ReplaceCheck { find: const_name.clone(), replace: check_param };
            replace.visit_type_mut(&mut ret_type);

            quote!(#ret_type)
        } else {
            let ret_type = &self.wrapped;

            quote!(#ret_type)
        }
    }

    /// Contains const generic in field type
    pub fn has_const_name(&self) -> bool {
        self.const_name.is_some()
    }
}

/// Fields of annotated structures
///
/// We only support single, unnamed field in enum variants and named structure 
/// fields. For named enum variants the inner definition should go in a separate
/// struct. For unnamed struct variants no useful composition function names can
/// be derived. Traits are not supported.
#[derive(Debug, Clone)]
pub enum ModelFields {
    Enum(Vec<(Ident, Option<ModelType>)>),
    Struct(Vec<(Ident, ModelType)>),
}

#[derive(Debug, Clone)]
pub struct Model {
    pub(crate) name: Ident,
    pub(crate) item: Item,
    pub(crate) fields: ModelFields,
    pub(crate) check_name: Ident,
    pub(crate) rem_param_types: Vec<GenericParam>,
}

impl Model {
    /// Return argument parameters of item
    pub(crate) fn param_args(&self) -> Punctuated<Ident, Comma> {
        self.rem_param_types.iter()
            .map(|x| match x {
                GenericParam::Type(t) => t.ident.clone(),
                GenericParam::Lifetime(l) => l.lifetime.ident.clone(),
                GenericParam::Const(c) => c.ident.clone(),
            })
            .collect()
    }

    /// Return item definition with filled-in checked parameter
    pub(crate) fn item_definition(&self, checked: Option<bool>) -> TokenStream {
        let check_param = match checked {
            Some(true) => Ident::new("true", Span::call_site()),
            Some(false) => Ident::new("false", Span::call_site()),
            None => self.check_name.clone(),
        };

        let (name, args) = (&self.name, self.param_args());
        quote!(#name<#check_param, #args>)
    }

    #[allow(dead_code)]
    pub(crate) fn visibility(&self) -> Visibility {
        match self.item {
            Item::Enum(ref obj) => obj.vis.clone(),
            Item::Struct(ref obj) => obj.vis.clone(),
            _ => unreachable!(),
        }
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
