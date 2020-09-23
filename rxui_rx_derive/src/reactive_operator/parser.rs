use quote::quote;
use syn::punctuated::Punctuated;
use syn::{DeriveInput, Token};

macro_rules! quote_and_parse {
    ($($t:tt)*) => (syn::parse(quote!{$($t)*}.into()).unwrap())
}

pub struct Parser<'a> {
    pub ast: &'a DeriveInput,
    pub fields: &'a Punctuated<syn::Field, Token![,]>,
}

impl<'a> Parser<'a> {
    pub fn from(ast: &'a syn::DeriveInput) -> Self {
        match ast.data {
            syn::Data::Struct(ref s) => match s.fields {
                syn::Fields::Named(ref fields) => Self {
                    ast: &ast,
                    fields: &fields.named,
                },
                _ => panic! {"#[derive(reactive_operator)] requires a C-style struct"},
            },
            _ => panic! {"#[derive(reactive_operator)] requires a C-style struct"},
        }
    }

    pub fn parse(&self) -> ParserOutput<'a> {
        let fields: Vec<_> = self.fields.iter().map(|field| Field::from(field)).collect();
        let sources: Vec<_> = fields
            .iter()
            .filter(|field| field.upstream_attr.is_some())
            .collect();
        let data_fields: Vec<_> = fields
            .iter()
            .filter(|field| field.upstream_attr.is_none() && field.reactive_operator_attr.is_none())
            .cloned()
            .collect();
        if sources.len() != 1 {
            panic! {"#[derive(reactive_operator)] requires exactly one field has to be marked as the source"};
        }
        let source = sources[0].clone();
        let upstream_attr = source.upstream_attr.as_ref().unwrap().clone();
        let where_clause = self
            .ast
            .generics
            .where_clause
            .as_ref()
            .expect("#[derive(reactive_operator)] requires a where clause");
        let upstream = UpstreamInfo::from(&source.ty, where_clause);
        ParserOutput {
            ast: self.ast,
            upstream_field: source,
            data_fields,
            settings: Settings {
                upstream,
                operator_ident: upstream_attr.operator_ident,
                trait_impls: upstream_attr.trait_impls,
                subscriber_ty: upstream_attr.subscriber_ty,
                subscription_ty: upstream_attr.subscription_ty,
                item_ty: upstream_attr.item_ty,
                error_ty: upstream_attr.error_ty,
            },
        }
    }
}

pub struct ParserOutput<'a> {
    pub ast: &'a DeriveInput,
    pub upstream_field: Field<'a>,
    pub data_fields: Vec<Field<'a>>,
    pub settings: Settings,
}

pub struct Settings {
    pub upstream: UpstreamInfo,
    pub operator_ident: Option<syn::Ident>,
    pub trait_impls: Option<TraitImpls>,
    pub subscriber_ty: Option<syn::Type>,
    pub subscription_ty: Option<syn::Type>,
    pub item_ty: Option<syn::Type>,
    pub error_ty: Option<syn::Type>,
}

#[derive(Clone)]
pub struct Field<'a> {
    pub ty: &'a syn::Type,
    pub ident: syn::Ident,
    upstream_attr: Option<UpstreamAttr>,
    reactive_operator_attr: Option<ReactiveOperatorAttr>,
}

impl<'a> Field<'a> {
    fn from(field: &'a syn::Field) -> Field<'a> {
        let (upstream_attr, reactive_operator_attr) = Self::parse_attrs(&field.attrs);
        Field {
            ty: &field.ty,
            ident: field.ident.clone().unwrap(),
            upstream_attr,
            reactive_operator_attr,
        }
    }

    fn parse_attrs(
        attrs: &'a [syn::Attribute],
    ) -> (Option<UpstreamAttr>, Option<ReactiveOperatorAttr>) {
        let a = Self::attr_by_name(attrs, "upstream").map(|attr| UpstreamAttr::from(attr));
        let b = Self::attr_by_name(attrs, "reactive_operator")
            .map(|attr| ReactiveOperatorAttr::from(attr));
        (a, b)
    }

    fn attr_by_name(attrs: &'a [syn::Attribute], name: &str) -> Option<&'a syn::Attribute> {
        for attr in attrs.iter() {
            match attr.style {
                syn::AttrStyle::Outer => {}
                _ => continue,
            }
            if Self::name_of_attr(attr) == name {
                return Some(attr);
            }
        }
        None
    }

    fn name_of_attr(attr: &syn::Attribute) -> &syn::Ident {
        let attr_name = attr
            .path
            .segments
            .iter()
            .last()
            .expect("#[derive(reactive_operator)] found empty path unexpectedly");
        &(*attr_name).ident
    }
}

#[derive(Default, Clone)]
struct UpstreamAttr {
    operator_ident: Option<syn::Ident>,
    trait_impls: Option<TraitImpls>,
    subscriber_ty: Option<syn::Type>,
    subscription_ty: Option<syn::Type>,
    item_ty: Option<syn::Type>,
    error_ty: Option<syn::Type>,
}

impl UpstreamAttr {
    pub fn from(attr: &syn::Attribute) -> Self {
        let list = as_meta_list(attr);
        let mut result = Self::default();
        for item in list.nested.iter() {
            match *item {
                syn::NestedMeta::Meta(syn::Meta::NameValue(ref kv)) => {
                    if let syn::Lit::Str(ref s) = kv.lit {
                        if kv.path.is_ident("subscriber") {
                            result.subscriber_ty =
                                Some(syn::parse_str(s.value().as_str()).unwrap());
                        } else if kv.path.is_ident("subscription") {
                            result.subscription_ty =
                                Some(syn::parse_str(s.value().as_str()).unwrap());
                        } else if kv.path.is_ident("item") {
                            result.item_ty = Some(syn::parse_str(s.value().as_str()).unwrap());
                        } else if kv.path.is_ident("error") {
                            result.error_ty = Some(syn::parse_str(s.value().as_str()).unwrap());
                        } else if kv.path.is_ident("derive_impls") {
                            result.trait_impls = Some(TraitImpls::from(s.value().as_str()));
                        } else if kv.path.is_ident("operator") {
                            result.operator_ident =
                                Some(syn::parse_str(s.value().as_str()).unwrap());
                        } else {
                            panic! {"#[derive(reactive_operator)] invalid key in reactive_operator attribute"};
                        }
                    } else {
                        panic! {"#[derive(reactive_operator)] non-string key"}
                    }
                }
                _ => panic! {"#[derive(reactive_operator)] expected only name-value pairs"},
            }
        }
        result
    }
}

#[derive(Default, Clone)]
pub struct TraitImpls {
    pub base: bool,
    pub local: bool,
    pub shared: bool,
}

impl TraitImpls {
    fn from(string: &str) -> Self {
        let mut result = Self::default();
        for elem in string.split_whitespace().map(|e| e.split(',')).flatten() {
            match elem {
                "base" => result.base = true,
                "local" => result.local = true,
                "shared" => result.shared = true,
                _ => panic! {"#[derive(reactive_operator)] unknown derive_impls element"},
            }
        }
        result
    }
}

#[derive(Clone)]
struct ReactiveOperatorAttr {
    ignore: bool,
}

impl ReactiveOperatorAttr {
    pub fn from(attr: &syn::Attribute) -> Self {
        let _ = as_meta_list(attr);
        Self { ignore: true }
    }
}

fn as_meta_list(attr: &syn::Attribute) -> syn::MetaList {
    let meta = match attr.parse_meta() {
        Ok(meta) => meta,
        Err(_) => {
            panic! {"#[derive(reactive_operator)] expects only a list of key-value pairs in attributes"}
        }
    };
    match meta {
        syn::Meta::List(l) => l,
        _ => {
            panic! {"#[derive(reactive_operator)] expects only a list of key-value pairs in attributes"}
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpstreamInfo {
    kind: UpstreamKind,
    supports_local: bool,
    supports_shared: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpstreamLocalInfo {
    kind: UpstreamKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpstreamSharedInfo {
    kind: UpstreamKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum UpstreamKind {
    Observable,
    Flow,
}

pub trait UpstreamImplTokens {
    fn impl_type(&self) -> proc_macro2::TokenStream;
    fn with_marker_tys(&self, ty: proc_macro2::TokenStream) -> proc_macro2::TokenStream;
    fn with_markers(&self, expr: proc_macro2::TokenStream) -> proc_macro2::TokenStream;
    fn marker_accessor(&self) -> proc_macro2::TokenStream;
    fn type_param_ident(&self) -> proc_macro2::TokenStream;
    fn downstream_trait(&self) -> proc_macro2::TokenStream;
    fn lifetime_impl_params(&self) -> Option<proc_macro2::TokenStream>;
    fn additional_bounds(&self) -> Option<proc_macro2::TokenStream>;
}

impl UpstreamInfo {
    fn new(kind: UpstreamKind, supports_local: bool, supports_shared: bool) -> Self {
        Self {
            kind,
            supports_local,
            supports_shared,
        }
    }

    fn from(source_ty: &syn::Type, where_clause: &syn::WhereClause) -> Self {
        for pred in where_clause.predicates.iter() {
            if let Some(pred) = super::try_get_predicate_for_type(pred, source_ty) {
                if Self::contains_bound(quote_and_parse! { core::Observable }, pred) {
                    return Self::new(UpstreamKind::Observable, true, true);
                }
                if Self::contains_bound(quote_and_parse! { core::LocalObservable<'o> }, pred) {
                    return Self::new(UpstreamKind::Observable, true, false);
                }
                if Self::contains_bound(quote_and_parse! { core::SharedObservable }, pred) {
                    return Self::new(UpstreamKind::Observable, false, true);
                }
                if Self::contains_bound(quote_and_parse! { core::Flow }, pred) {
                    return Self::new(UpstreamKind::Flow, true, true);
                }
                if Self::contains_bound(quote_and_parse! { core::LocalFlow<'o> }, pred) {
                    return Self::new(UpstreamKind::Flow, true, false);
                }
                if Self::contains_bound(quote_and_parse! { core::SharedFlow }, pred) {
                    return Self::new(UpstreamKind::Flow, false, true);
                }
            }
        }
        panic! {"#[derive(reactive_operator)] unknown upstream type"};
    }

    fn contains_bound(path: syn::Path, pred: &syn::PredicateType) -> bool {
        for bound in pred.bounds.iter() {
            match bound {
                syn::TypeParamBound::Trait(bound) => {
                    let bound_path = &bound.path;
                    let bound_path = quote! { #bound_path };
                    let path = quote! { #path };
                    return bound_path.to_string() == path.to_string();
                }
                _ => continue,
            }
        }
        false
    }

    pub fn as_local(&self) -> Option<UpstreamLocalInfo> {
        if self.supports_local {
            Some(UpstreamLocalInfo {
                kind: self.kind.clone(),
            })
        } else {
            None
        }
    }

    pub fn as_shared(&self) -> Option<UpstreamSharedInfo> {
        if self.supports_shared {
            Some(UpstreamSharedInfo {
                kind: self.kind.clone(),
            })
        } else {
            None
        }
    }

    pub fn base_trait(&self) -> proc_macro2::TokenStream {
        if self.kind.is_observable() {
            quote! {core::Observable}
        } else {
            quote! {core::Flow}
        }
    }
}

impl UpstreamKind {
    fn is_observable(&self) -> bool {
        *self == UpstreamKind::Observable
    }
}

impl UpstreamImplTokens for UpstreamLocalInfo {
    fn impl_type(&self) -> proc_macro2::TokenStream {
        if self.kind.is_observable() {
            quote! {core::LocalObservable}
        } else {
            quote! {core::LocalFlow}
        }
    }

    fn with_marker_tys(&self, ty: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        if self.kind.is_observable() {
            quote! {crate::marker::Observable<#ty>}
        } else {
            quote! {crate::marker::Flow<#ty>}
        }
    }

    fn with_markers(&self, expr: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        if self.kind.is_observable() {
            quote! {crate::marker::Observable::new(#expr)}
        } else {
            quote! {crate::marker::Flow::new(#expr)}
        }
    }

    fn marker_accessor(&self) -> proc_macro2::TokenStream {
        quote! {.actual}
    }

    fn type_param_ident(&self) -> proc_macro2::TokenStream {
        if self.kind.is_observable() {
            quote! {Cancellable}
        } else {
            quote! {Subscription}
        }
    }

    fn downstream_trait(&self) -> proc_macro2::TokenStream {
        if self.kind.is_observable() {
            quote! {core::Observer}
        } else {
            quote! {core::Subscriber}
        }
    }

    fn lifetime_impl_params(&self) -> Option<proc_macro2::TokenStream> {
        Some(quote! {'o})
    }

    fn additional_bounds(&self) -> Option<proc_macro2::TokenStream> {
        None
    }
}

impl UpstreamImplTokens for UpstreamSharedInfo {
    fn impl_type(&self) -> proc_macro2::TokenStream {
        if self.kind.is_observable() {
            quote! {core::SharedObservable}
        } else {
            quote! {core::SharedFlow}
        }
    }

    fn with_marker_tys(&self, ty: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        if self.kind.is_observable() {
            quote! {crate::marker::Shared<crate::marker::Observable<#ty>>}
        } else {
            quote! {crate::marker::Shared<crate::marker::Flow<#ty>>}
        }
    }

    fn with_markers(&self, expr: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        if self.kind.is_observable() {
            quote! {crate::marker::Shared::new(crate::marker::Observable::new(#expr))}
        } else {
            quote! {crate::marker::Shared::new(crate::marker::Flow::new(#expr))}
        }
    }

    fn marker_accessor(&self) -> proc_macro2::TokenStream {
        quote! {.actual.actual}
    }

    fn type_param_ident(&self) -> proc_macro2::TokenStream {
        if self.kind.is_observable() {
            quote! {Cancellable}
        } else {
            quote! {Subscription}
        }
    }

    fn downstream_trait(&self) -> proc_macro2::TokenStream {
        if self.kind.is_observable() {
            quote! {core::Observer}
        } else {
            quote! {core::Subscriber}
        }
    }

    fn lifetime_impl_params(&self) -> Option<proc_macro2::TokenStream> {
        None
    }

    fn additional_bounds(&self) -> Option<proc_macro2::TokenStream> {
        Some(quote! {+ Send + 'static})
    }
}
