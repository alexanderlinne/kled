use proc_macro::TokenStream;
use quote::quote;
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
    derive_struct(&Parser::from(&ast).parse()).into()
}

fn derive_struct(output: &ParserOutput<'_>) -> proc_macro2::TokenStream {
    let base = generate_base_impl(output);
    let local = if let Some(local_info) = output.settings.upstream.as_local() {
        generate_actual_subscribe_impl(&output, local_info)
    } else {
        TokenStream::default().into()
    };
    let shared = if let Some(shared_info) = output.settings.upstream.as_shared() {
        generate_actual_subscribe_impl(&output, shared_info)
    } else {
        TokenStream::default().into()
    };
    quote! {#base #local #shared}
}

fn generate_base_impl(output: &ParserOutput<'_>) -> proc_macro2::TokenStream {
    let name = &output.ast.ident;
    let (impl_generics, ty_generics, where_clause) = output.ast.generics.split_for_impl();
    let base_trait = output.settings.upstream.base_trait();
    let upstream_type = output.upstream_field.ty;
    let item_ty = match &output.settings.item_ty {
        Some(ty) => ty.clone(),
        None => quote_and_parse! { #upstream_type::Item },
    };
    let error_ty = match &output.settings.error_ty {
        Some(ty) => ty.clone(),
        None => quote_and_parse! { #upstream_type::Error },
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

fn generate_actual_subscribe_impl<Tokens>(
    output: &ParserOutput<'_>,
    tokens: Tokens,
) -> proc_macro2::TokenStream
where
    Tokens: UpstreamImplTokens,
{
    let impl_type = tokens.impl_type();
    let type_param_ident = tokens.type_param_ident();
    let downstream_trait = tokens.downstream_trait();
    let lifetime_impl_params = tokens.lifetime_impl_params();
    let additional_bounds = tokens.additional_bounds();

    let name = &output.ast.ident;
    let upstream_type = output.upstream_field.ty;
    let generic_params = &output.ast.generics.params;
    let (_, ty_generics, where_clause) = output.ast.generics.split_for_impl();
    let where_preds = remove_predicates_for_type(
        &output.upstream_field.ty,
        &where_clause.as_ref().expect("").predicates,
    );
    let upstream_name = &output.upstream_field.ident;
    let downstream_ty = &output.settings.downstream_ty;
    let subscription_ty = match &output.settings.subscription_ty {
        Some(subscription_ty) => quote! { #subscription_ty },
        None => quote! {#upstream_type :: #type_param_ident},
    };
    let field_idents = output.data_fields.iter().map(|f| &f.ident);
    let lifetime_impl_params_with_lg = lifetime_impl_params.as_ref().map(|t| quote! {< #t >});
    let lifetime_impl_params_with_plus = lifetime_impl_params.as_ref().map(|t| quote! {+ #t});
    let lifetime_impl_params = if contains_o_lifetime(&output.ast.generics) {
        None
    } else {
        lifetime_impl_params
    };
    let lifetime_impl_params_with_comma = lifetime_impl_params.as_ref().map(|t| quote! {#t,});
    quote! {
        impl<#lifetime_impl_params_with_comma #generic_params> #impl_type #lifetime_impl_params_with_lg
            for #name #ty_generics
        where
            #upstream_type: #impl_type #lifetime_impl_params_with_lg,
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

fn is_predicate_for_type(ty: &syn::Type, pred: &syn::WherePredicate) -> bool {
    try_get_predicate_for_type(pred, ty).is_some()
}

fn contains_o_lifetime(generics: &syn::Generics) -> bool {
    for lifetime in generics.lifetimes() {
        if lifetime.lifetime.ident == "o" {
            return true;
        }
    }
    false
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
