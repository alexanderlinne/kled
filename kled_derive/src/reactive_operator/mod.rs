use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_macro_input, DeriveInput};

mod parser;
use parser::*;

macro_rules! quote_and_parse {
    ($($t:tt)*) => (syn::parse(quote!{$($t)*}.into()).unwrap())
}

pub fn derive(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    generate_trait_impl(&Parser::from(&ast).parse()).into()
}

fn generate_trait_impl(output: &ParserOutput<'_>) -> proc_macro2::TokenStream {
    let tokens = &output.settings.upstream;
    let base_trait = tokens.base_trait();
    let type_param_ident = tokens.type_param_ident();
    let downstream_trait = tokens.downstream_trait();

    let struct_ident = &output.ast.ident;
    let upstream_ty = output.upstream_field.ty;
    let item_ty = match &output.settings.item_ty {
        Some(ty) => ty.clone(),
        None => quote_and_parse! { #upstream_ty::Item },
    };
    let error_ty = match &output.settings.error_ty {
        Some(ty) => ty.clone(),
        None => quote_and_parse! { #upstream_ty::Error },
    };
    let generic_params = &output.ast.generics.params;
    let (_, ty_generics, where_clause) = output.ast.generics.split_for_impl();
    let where_preds = remove_predicates_for_type(
        &output.upstream_field.ty,
        &where_clause.as_ref().expect("").predicates,
    );
    let missing_clauses = generate_missing_where_clauses(output);
    let upstream_name = &output.upstream_field.ident;
    let subscriber_ty = match &output.settings.subscriber_ty {
        Some(ty) => ty.clone(),
        None => derive_subscriber_ty(struct_ident),
    };
    let subscription_ty = match &output.settings.subscription_ty {
        Some(subscription_ty) => quote! { #subscription_ty },
        None => quote! {#upstream_ty :: #type_param_ident},
    };
    let field_idents = output.data_fields.iter().map(|f| &f.ident);
    quote! {
        impl<#generic_params> #base_trait
            for #struct_ident #ty_generics
        where
            #upstream_ty: #base_trait,
            #(#where_preds + Send + 'static),*
            #missing_clauses
        {
            type Item = #item_ty;
            type Error = #error_ty;
            type #type_param_ident = #subscription_ty;

            fn actual_subscribe<Downstream>(self, downstream: Downstream)
            where
                Downstream: #downstream_trait<Self::#type_param_ident, Self::Item, Self::Error> + Send + 'static,
            {
                self. #upstream_name .actual_subscribe( #subscriber_ty ::new(
                    downstream,
                    #( self. #field_idents),*
                ));
            }
        }
    }
}

fn generate_missing_where_clauses(output: &ParserOutput<'_>) -> Option<proc_macro2::TokenStream> {
    let covered_idents: Vec<_> = output
        .ast
        .generics
        .where_clause
        .as_ref()
        .unwrap()
        .predicates
        .iter()
        .filter_map(|pred| {
            if let syn::WherePredicate::Type(pred) = pred {
                let ty = &pred.bounded_ty;
                Some(quote! {#ty}.to_string())
            } else {
                None
            }
        })
        .collect();
    let missing_clauses: Vec<_> = output
        .ast
        .generics
        .type_params()
        .map(|param| param.ident.to_string())
        .filter_map(|ident| {
            if !covered_idents.contains(&ident) {
                let ident = format_ident! {"{}", ident};
                Some(quote! {#ident: Send + 'static})
            } else {
                None
            }
        })
        .collect();
    if missing_clauses.is_empty() {
        None
    } else {
        Some(quote! {,#( #missing_clauses),*})
    }
}

fn derive_subscriber_ty(ident: &proc_macro2::Ident) -> syn::Type {
    let ident = ident.to_string();
    if let Some(ident) = ident.strip_prefix("Observable") {
        syn::parse_str(format!("{}Observer", ident).as_str()).unwrap()
    } else if let Some(ident) = ident.strip_prefix("Flow") {
        syn::parse_str(format!("{}Subscriber", ident).as_str()).unwrap()
    } else {
        panic! {"#[derive(reactive_operator)] unexpected struct identifier"};
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

fn is_predicate_for_type(ty: &syn::Type, pred: &syn::WherePredicate) -> bool {
    try_get_predicate_for_type(pred, ty).is_some()
}

fn try_get_predicate_for_type<'a>(
    pred: &'a syn::WherePredicate,
    ty: &syn::Type,
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
