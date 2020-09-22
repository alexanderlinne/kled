use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_macro_input, DeriveInput, Token};

macro_rules! quote_and_parse {
    ($($t:tt)*) => (syn::parse(quote!{$($t)*}.into()).unwrap())
}

pub fn derive(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    match ast.data {
        syn::Data::Struct(ref s) => match s.fields {
            syn::Fields::Named(ref fields) => derive_struct(&ast, &fields.named),
            _ => panic! {"Expected named fields"},
        },
        _ => panic! {"Expected a struct"},
    }
    .into()
}

fn derive_struct(
    ast: &syn::DeriveInput,
    fields: &syn::punctuated::Punctuated<syn::Field, Token![,]>,
) -> proc_macro2::TokenStream {
    let where_clause = ast
        .generics
        .where_clause
        .as_ref()
        .expect("struct must have a where clause");
    let sources: Vec<_> = fields
        .iter()
        .filter_map(|f| try_parse_upstream_field(f, where_clause))
        .collect();
    let data_fields: Vec<_> = fields.iter().filter_map(|f| try_parse_field(f)).collect();
    if sources.len() != 1 {
        panic! {"Exactly one field has to be marked as the source"};
    }
    let source = &sources[0];
    if source.kind.is_observable() {
        derive_observable(&ast, &source, &data_fields)
    } else {
        derive_flow(&ast, &source, &data_fields)
    }
}

fn try_parse_field<'a>(field: &'a syn::Field) -> Option<Field<'a>> {
    match get_attr_by_name(&field.attrs, "upstream") {
        Some(_) => None,
        None => parse_field(field),
    }
}

fn parse_field<'a>(field: &'a syn::Field) -> Option<Field<'a>> {
    match get_attr_by_name(&field.attrs, "reactive_operator") {
        Some(_) => None,
        None => Some(Field::new(field)),
    }
}

fn try_parse_upstream_field<'a>(
    field: &'a syn::Field,
    where_clause: &syn::WhereClause,
) -> Option<UpstreamField<'a>> {
    match get_attr_by_name(&field.attrs, "upstream") {
        Some(attr) => Some(UpstreamField::new(field, where_clause, attr)),
        None => None,
    }
}

fn get_attr_by_name<'a>(attrs: &'a [syn::Attribute], name: &str) -> Option<&'a syn::Attribute> {
    for attr in attrs.iter() {
        match attr.style {
            syn::AttrStyle::Outer => {}
            _ => continue,
        }
        if name_of_attr(attr) == name {
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
        .expect("Expected at least one segment in attribute path");
    &(*attr_name).ident
}

fn derive_observable(
    ast: &syn::DeriveInput,
    source: &UpstreamField,
    data_fields: &Vec<Field>,
) -> proc_macro2::TokenStream {
    let observable = generate_observable_impl(&ast, &source);
    let local = if source.kind.supports_local() {
        generate_local_observable_impl(&ast, &source, &data_fields)
    } else {
        TokenStream::default().into()
    };
    let shared = if source.kind.supports_shared() {
        generate_shared_observable_impl(&ast, &source, &data_fields)
    } else {
        TokenStream::default().into()
    };
    quote! {#observable #local #shared}
}

fn derive_flow(
    ast: &syn::DeriveInput,
    source: &UpstreamField,
    data_fields: &Vec<Field>,
) -> proc_macro2::TokenStream {
    let observable = generate_flow_impl(&ast, &source);
    let local = if source.kind.supports_local() {
        generate_local_flow_impl(&ast, &source, &data_fields)
    } else {
        TokenStream::default().into()
    };
    let shared = if source.kind.supports_shared() {
        generate_shared_flow_impl(&ast, &source, &data_fields)
    } else {
        TokenStream::default().into()
    };
    quote! {#observable #local #shared}
}

fn generate_observable_impl(
    ast: &syn::DeriveInput,
    source: &UpstreamField,
) -> proc_macro2::TokenStream {
    generate_base_impl(ast, source, quote! {core::Observable})
}

fn generate_flow_impl(ast: &syn::DeriveInput, source: &UpstreamField) -> proc_macro2::TokenStream {
    generate_base_impl(ast, source, quote! {core::Flow})
}

fn generate_base_impl(
    ast: &syn::DeriveInput,
    source: &UpstreamField,
    base_trait: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let source_type = source.field.ty;
    let item_ty = match &source.settings.item_ty {
        Some(ty) => ty.clone(),
        None => quote_and_parse! { #source_type::Item },
    };
    let error_ty = match &source.settings.error_ty {
        Some(ty) => ty.clone(),
        None => quote_and_parse! { #source_type::Error },
    };
    quote! {
        impl #impl_generics #base_trait
            for #name #ty_generics
        #where_clause
        {
            type Item = #item_ty;
            type Error = #error_ty;
        }
    }
}

fn generate_local_observable_impl(
    ast: &syn::DeriveInput,
    source: &UpstreamField,
    data_fields: &Vec<Field>,
) -> proc_macro2::TokenStream {
    generate_actual_subscribe_impl(
        ast,
        source,
        data_fields,
        quote! {core::LocalObservable},
        quote! {Cancellable},
        quote! {core::Observer},
        Some(quote! {'o}),
        None,
    )
}

fn generate_shared_observable_impl(
    ast: &syn::DeriveInput,
    source: &UpstreamField,
    data_fields: &Vec<Field>,
) -> proc_macro2::TokenStream {
    generate_actual_subscribe_impl(
        ast,
        source,
        data_fields,
        quote! {core::SharedObservable},
        quote! {Cancellable},
        quote! {core::Observer},
        None,
        Some(quote! {+ Send + 'static}),
    )
}

fn generate_local_flow_impl(
    ast: &syn::DeriveInput,
    source: &UpstreamField,
    data_fields: &Vec<Field>,
) -> proc_macro2::TokenStream {
    generate_actual_subscribe_impl(
        ast,
        source,
        data_fields,
        quote! {core::LocalFlow},
        quote! {Subscription},
        quote! {core::Subscriber},
        Some(quote! {'o}),
        None,
    )
}

fn generate_shared_flow_impl(
    ast: &syn::DeriveInput,
    source: &UpstreamField,
    data_fields: &Vec<Field>,
) -> proc_macro2::TokenStream {
    generate_actual_subscribe_impl(
        ast,
        source,
        data_fields,
        quote! {core::SharedFlow},
        quote! {Subscription},
        quote! {core::Subscriber},
        None,
        Some(quote! {+ Send + 'static}),
    )
}

fn generate_actual_subscribe_impl(
    ast: &syn::DeriveInput,
    source: &UpstreamField,
    data_fields: &Vec<Field>,
    impl_type: proc_macro2::TokenStream,
    type_param_ident: proc_macro2::TokenStream,
    downstream_trait: proc_macro2::TokenStream,
    lifetime_impl_params: Option<proc_macro2::TokenStream>,
    additional_bounds: Option<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let source_type = source.field.ty;
    let generic_params = &ast.generics.params;
    let (_, ty_generics, where_clause) = ast.generics.split_for_impl();
    let where_preds = remove_predicates_for_type(
        &source.field.ty,
        &where_clause.as_ref().expect("").predicates,
    );
    let upstream_name = &source.field.ident;
    let downstream_ty = &source.settings.downstream_ty;
    let subscription_ty = match &source.settings.subscription_ty {
        Some(subscription_ty) => quote! { #subscription_ty },
        None => quote! {#source_type :: #type_param_ident},
    };
    let field_idents = data_fields.iter().map(|f| &f.ident);
    let lifetime_impl_params_with_lg = lifetime_impl_params.as_ref().map(|t| quote! {< #t >});
    let lifetime_impl_params_with_plus = lifetime_impl_params.as_ref().map(|t| quote! {+ #t});
    let lifetime_impl_params = if contains_o_lifetime(&ast.generics) {
        None
    } else {
        lifetime_impl_params
    };
    let lifetime_impl_params_with_comma = lifetime_impl_params.as_ref().map(|t| quote! {#t,});
    quote! {
        impl<#lifetime_impl_params_with_comma #generic_params> #impl_type #lifetime_impl_params_with_lg
            for #name #ty_generics
        where
            #source_type: #impl_type #lifetime_impl_params_with_lg,
            #(#where_preds #lifetime_impl_params_with_plus #additional_bounds),*
        {
            type #type_param_ident = #subscription_ty;

            fn actual_subscribe<Downstream>(self, downstream: Downstream)
            where
                Downstream: #downstream_trait<Self::#type_param_ident, Self::Item, Self::Error> #lifetime_impl_params_with_plus #additional_bounds,
            {
                self. #upstream_name .actual_subscribe( #downstream_ty ::new(
                    downstream,
                    #( self. #field_idents),*
                ));
            }
        }
    }
}

fn remove_predicates_for_type<'a>(
    ty: &syn::Type,
    preds: &'a Punctuated<syn::WherePredicate, Comma>,
) -> Vec<&'a syn::WherePredicate> {
    let mut result = Vec::new();
    for pred in preds.iter() {
        if !is_predicate_for_type(ty, pred) {
            result.push(pred);
        }
    }
    result
}

fn contains_o_lifetime(generics: &syn::Generics) -> bool {
    for lifetime in generics.lifetimes() {
        if lifetime.lifetime.ident.to_string() == "o" {
            return true;
        }
    }
    false
}

struct Field<'a> {
    ty: &'a syn::Type,
    ident: syn::Ident,
}

impl<'a> Field<'a> {
    fn new(field: &'a syn::Field) -> Field<'a> {
        Field {
            ty: &field.ty,
            ident: field.ident.clone().unwrap(),
        }
    }
}

struct UpstreamField<'a> {
    field: Field<'a>,
    kind: UpstreamKind,
    settings: Settings,
}

impl<'a> UpstreamField<'a> {
    fn new(
        field: &'a syn::Field,
        where_clause: &syn::WhereClause,
        attr: &syn::Attribute,
    ) -> UpstreamField<'a> {
        let meta = match attr.parse_meta() {
            Ok(meta) => meta,
            Err(_) => panic! {"rx_operator attribute has wrong format"},
        };
        let list = match meta {
            syn::Meta::List(l) => l,
            _ => panic! {"rx_operator attribute has wrong format"},
        };
        UpstreamField {
            field: Field::new(field),
            kind: UpstreamKind::from(&field.ty, where_clause),
            settings: Settings::from(list),
        }
    }
}

struct Settings {
    downstream_ty: Option<syn::Type>,
    subscription_ty: Option<syn::Type>,
    item_ty: Option<syn::Type>,
    error_ty: Option<syn::Type>,
}

impl Settings {
    pub fn from(list: syn::MetaList) -> Settings {
        let mut result = Settings {
            downstream_ty: None,
            subscription_ty: None,
            item_ty: None,
            error_ty: None,
        };
        for item in list.nested.iter() {
            match *item {
                syn::NestedMeta::Meta(syn::Meta::NameValue(ref kv)) => {
                    if let syn::Lit::Str(ref s) = kv.lit {
                        if kv.path.is_ident("downstream") {
                            result.downstream_ty =
                                Some(syn::parse_str(s.value().as_str()).unwrap());
                        } else if kv.path.is_ident("subscription") {
                            result.subscription_ty =
                                Some(syn::parse_str(s.value().as_str()).unwrap());
                        } else if kv.path.is_ident("item") {
                            result.item_ty = Some(syn::parse_str(s.value().as_str()).unwrap());
                        } else if kv.path.is_ident("error") {
                            result.error_ty = Some(syn::parse_str(s.value().as_str()).unwrap());
                        } else {
                            panic! {"Invalid key in rx_observable attribute"};
                        }
                    } else {
                        panic! {"Found non-string key"}
                    }
                }
                _ => panic! {"Expected only name-value pairs"},
            }
        }
        result
    }
}

#[derive(Debug, PartialEq, Eq)]
enum UpstreamKind {
    Observable,
    LocalObservable,
    SharedObservable,
    Flow,
    LocalFlow,
    SharedFlow,
}

impl UpstreamKind {
    fn from(source_ty: &syn::Type, where_clause: &syn::WhereClause) -> Self {
        for pred in where_clause.predicates.iter() {
            if let Some(pred) = try_get_predicate_type(source_ty, pred) {
                if Self::contains_bound(quote_and_parse! { core::Observable }, pred) {
                    return Self::Observable;
                }
                if Self::contains_bound(quote_and_parse! { core::LocalObservable<'o> }, pred) {
                    return Self::LocalObservable;
                }
                if Self::contains_bound(quote_and_parse! { core::SharedObservable }, pred) {
                    return Self::SharedObservable;
                }
                if Self::contains_bound(quote_and_parse! { core::Flow }, pred) {
                    return Self::Flow;
                }
                if Self::contains_bound(quote_and_parse! { core::LocalFlow<'o> }, pred) {
                    return Self::LocalFlow;
                }
                if Self::contains_bound(quote_and_parse! { core::SharedFlow }, pred) {
                    return Self::SharedFlow;
                }
            }
        }
        panic! {"Couldn't identify source type"};
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

    fn is_observable(&self) -> bool {
        *self == Self::Observable
            || *self == Self::LocalObservable
            || *self == Self::SharedObservable
    }

    fn supports_local(&self) -> bool {
        *self == Self::Observable
            || *self == Self::Flow
            || *self == Self::LocalObservable
            || *self == Self::LocalFlow
    }

    fn supports_shared(&self) -> bool {
        *self == Self::Observable
            || *self == Self::Flow
            || *self == Self::SharedObservable
            || *self == Self::SharedFlow
    }
}

fn is_predicate_for_type(ty: &syn::Type, pred: &syn::WherePredicate) -> bool {
    try_get_predicate_type(ty, pred).is_some()
}

fn try_get_predicate_type<'a>(
    ty: &syn::Type,
    pred: &'a syn::WherePredicate,
) -> Option<&'a syn::PredicateType> {
    match pred {
        syn::WherePredicate::Type(pred) => {
            let bounded_ty = &pred.bounded_ty;
            let ty = quote! { #ty };
            let bounded_ty = quote! { #bounded_ty };
            if ty.to_string() == bounded_ty.to_string() {
                Some(pred)
            } else {
                None
            }
        }
        _ => None,
    }
}
