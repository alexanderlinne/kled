use inflector::Inflector;
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
    derive_struct(&Parser::from(&ast).parse()).into()
}

fn derive_struct(output: &ParserOutput<'_>) -> proc_macro2::TokenStream {
    if let Some(trait_impls) = &output.settings.trait_impls {
        let base = if trait_impls.base {
            generate_base_trait_impl(&output)
        } else {
            TokenStream::default().into()
        };
        let local = if trait_impls.local {
            derive_local_trait_impl(&output)
        } else {
            TokenStream::default().into()
        };
        let shared = if trait_impls.shared {
            derive_shared_trait_impl(&output)
        } else {
            TokenStream::default().into()
        };
        let local_operator = derive_local_operator_impl(&output);
        let shared_operator = derive_shared_operator_impl(&output);
        quote! {#base #local #shared #local_operator #shared_operator}
    } else {
        let base = generate_base_trait_impl(&output);
        let local = derive_local_trait_impl(&output);
        let shared = derive_shared_trait_impl(&output);
        let local_operator = derive_local_operator_impl(&output);
        let shared_operator = derive_shared_operator_impl(&output);
        quote! {#base #local #shared #local_operator #shared_operator}
    }
}

fn derive_local_trait_impl(output: &ParserOutput<'_>) -> proc_macro2::TokenStream {
    if let Some(local_info) = output.settings.upstream.as_local() {
        generate_actual_subscribe_impl(&output, local_info)
    } else {
        TokenStream::default().into()
    }
}

fn derive_shared_trait_impl(output: &ParserOutput<'_>) -> proc_macro2::TokenStream {
    if let Some(shared_info) = output.settings.upstream.as_shared() {
        generate_actual_subscribe_impl(&output, shared_info)
    } else {
        TokenStream::default().into()
    }
}

fn derive_local_operator_impl(output: &ParserOutput<'_>) -> proc_macro2::TokenStream {
    if let Some(local_info) = output.settings.upstream.as_local() {
        generate_operator_impl(&output, local_info)
    } else {
        TokenStream::default().into()
    }
}

fn derive_shared_operator_impl(output: &ParserOutput<'_>) -> proc_macro2::TokenStream {
    if let Some(shared_info) = output.settings.upstream.as_shared() {
        generate_operator_impl(&output, shared_info)
    } else {
        TokenStream::default().into()
    }
}

fn generate_base_trait_impl(output: &ParserOutput<'_>) -> proc_macro2::TokenStream {
    let struct_ident = &output.ast.ident;
    let (impl_generics, ty_generics, where_clause) = output.ast.generics.split_for_impl();
    let base_trait = output.settings.upstream.base_trait();
    let upstream_ty = output.upstream_field.ty;
    let item_ty = match &output.settings.item_ty {
        Some(ty) => ty.clone(),
        None => quote_and_parse! { #upstream_ty::Item },
    };
    let error_ty = match &output.settings.error_ty {
        Some(ty) => ty.clone(),
        None => quote_and_parse! { #upstream_ty::Error },
    };
    quote! {
        impl #impl_generics #base_trait
            for #struct_ident #ty_generics
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

    let struct_ident = &output.ast.ident;
    let upstream_ty = output.upstream_field.ty;
    let generic_params = &output.ast.generics.params;
    let (_, ty_generics, where_clause) = output.ast.generics.split_for_impl();
    let where_preds = remove_predicates_for_type(
        &output.upstream_field.ty,
        &where_clause.as_ref().expect("").predicates,
    );
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
            for #struct_ident #ty_generics
        where
            #upstream_ty: #impl_type #lifetime_impl_params_with_lg,
            #(#where_preds #lifetime_impl_params_with_plus #additional_bounds),*
        {
            type #type_param_ident = #subscription_ty;

            fn actual_subscribe<Downstream>(self, downstream: Downstream)
            where
                Downstream: #downstream_trait<Self::#type_param_ident, Self::Item, Self::Error> #lifetime_impl_params_with_plus #additional_bounds,
            {
                self. #upstream_name .actual_subscribe( #subscriber_ty ::new(
                    downstream,
                    #( self. #field_idents),*
                ));
            }
        }
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

fn generate_operator_impl<Tokens>(
    output: &ParserOutput<'_>,
    tokens: Tokens,
) -> proc_macro2::TokenStream
where
    Tokens: UpstreamImplTokens,
{
    let impl_type = tokens.impl_type();
    let marker_accessor = tokens.marker_accessor();
    let lifetime_impl_params = tokens.lifetime_impl_params();
    let additional_bounds = tokens.additional_bounds();

    let struct_ident = &output.ast.ident;
    let operator_ident = match &output.settings.operator_ident {
        Some(ident) => quote! {#ident},
        None => derive_operator_name(struct_ident),
    };
    let upstream_ty = output.upstream_field.ty;
    let upstream_marker_ty = tokens.with_marker_tys(quote! {#upstream_ty});
    let generic_params_tail: Vec<_> = output.ast.generics.type_params().skip(1).cloned().collect();
    let (_, ty_generics, where_clause) = output.ast.generics.split_for_impl();
    let result_ty = tokens.with_marker_tys(quote! {#struct_ident #ty_generics});
    let where_preds = remove_predicates_for_type(
        &output.upstream_field.ty,
        &where_clause.as_ref().expect("").predicates,
    );
    let field_idents = output.data_fields.iter().map(|f| &f.ident);
    let arguments = output.data_fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote! {#ident: #ty}
    });
    let lifetime_impl_params_with_lg = lifetime_impl_params.as_ref().map(|t| quote! {< #t >});
    let lifetime_impl_params_with_plus = lifetime_impl_params.as_ref().map(|t| quote! {+ #t});
    let lifetime_impl_params_with_comma = lifetime_impl_params.as_ref().map(|t| quote! {#t,});
    let result_expr = tokens
        .with_markers(quote! {#struct_ident ::new( self #marker_accessor, #(#field_idents),* )});
    quote! {
        impl<#upstream_ty> #upstream_marker_ty {
            pub fn #operator_ident<#lifetime_impl_params_with_comma #(#generic_params_tail),*>(
                self,
                #(#arguments),*
            ) -> #result_ty
            where
                #upstream_ty: #impl_type #lifetime_impl_params_with_lg + Sized,
                #(#where_preds #lifetime_impl_params_with_plus #additional_bounds),*
            {
                #result_expr
            }
        }
    }
}

fn derive_operator_name(ident: &proc_macro2::Ident) -> proc_macro2::TokenStream {
    let ident = ident.to_string();
    if let Some(ident) = ident.strip_prefix("Observable") {
        let ident = format_ident! {"{}", ident.to_snake_case()};
        quote! {#ident}
    } else if let Some(ident) = ident.strip_prefix("Flow") {
        let ident = format_ident! {"{}", ident.to_snake_case()};
        quote! {#ident}
    } else {
        panic! {"#[derive(reactive_operator)] unexpected struct identifier"};
    }
}
