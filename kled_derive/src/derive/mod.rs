use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, parse_str, Fields, Ident, ItemStruct};

#[derive(Clone, Debug, PartialEq, Eq, FromMeta)]
#[darling(default)]
pub enum UpstreamTy {
    Observable,
    Flow,
}

#[derive(FromMeta)]
pub struct Args {
    #[darling(rename = "type")]
    upstream: UpstreamTy,
    #[darling(default)]
    subscriber: Option<syn::LitStr>,
    #[darling(default)]
    subscription: Option<syn::LitStr>,
    #[darling(default)]
    item: Option<syn::Ident>,
    #[darling(default)]
    error: Option<syn::Ident>,
}

impl Args {
    fn subscriber(&self) -> Option<syn::Type> {
        self.subscriber
            .as_ref()
            .map(|s| match parse_str(&s.value()) {
                Ok(ty) => ty,
                Err(_) => abort!(s, "`#[operator]` subscriber must be a valid type"),
            })
    }

    fn subscriber_trait(&self) -> Ident {
        match self.upstream {
            UpstreamTy::Flow => format_ident!("Subscriber"),
            UpstreamTy::Observable => format_ident!("Observer"),
        }
    }

    fn subscription(&self) -> syn::Type {
        self.subscription
            .as_ref()
            .map(|s| match parse_str(&s.value()) {
                Ok(ty) => ty,
                Err(_) => abort!(s, "`#[operator]` subscription must be a valid type"),
            })
            .unwrap_or(match self.upstream {
                UpstreamTy::Flow => parse_quote! {Subscription},
                UpstreamTy::Observable => parse_quote! {Cancellable},
            })
    }

    fn item(&self) -> syn::Ident {
        self.item
            .as_ref()
            .cloned()
            .unwrap_or_else(|| parse_quote! {Item})
    }

    fn error(&self) -> syn::Ident {
        self.error
            .as_ref()
            .cloned()
            .unwrap_or_else(|| parse_quote! {Error})
    }
}

pub fn derive(args: &Args, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);
    let mut result = quote! {};
    result.extend(derive_operator_struct(&args, &item));
    result.into()
}

fn derive_operator_struct(args: &Args, item: &ItemStruct) -> proc_macro2::TokenStream {
    let attrs = &item.attrs;
    let vis = &item.vis;
    let ident = &item.ident;
    let upstream_ty = upstream_ty_from(&args);
    let subscription_ty = subscription_ty_from(&args);
    let generic_params = &item.generics.params;
    let generic_params_iter = item.generics.params.iter();
    let (_, _, where_clause) = item.generics.split_for_impl();
    let predicates = where_clause.map(|where_clause| &where_clause.predicates);
    let fields = match &item.fields {
        Fields::Unnamed(fields) => abort!(
            fields,
            "`#[operator]` deriving from structs with unnamed fields is not yet implemented"
        ),
        fields => fields.iter(),
    };
    let subscriber_trait = args.subscriber_trait();
    let downstream_params = downstream_params(&args);
    let subscriber_ident = args
        .subscriber()
        .unwrap_or_else(|| subscriber_ident_from(&args, &ident));
    let field_idents = fields.clone().map(|f| &f.ident);
    quote! {
        #(#attrs)*
        #[derive(new)]
        #vis struct #ident<#upstream_ty, #subscription_ty, Item, Error, #generic_params>
        #where_clause
        {
            upstream: #upstream_ty,
            #(#fields,)*
            phantom: ::std::marker::PhantomData<(#upstream_ty, #subscription_ty, Item, Error, #generic_params)>,
        }

        #[automatically_derived]
        #[async_trait::async_trait]
        impl<#upstream_ty, #subscription_ty, Item, Error, #generic_params>
            core::#upstream_ty<#downstream_params>
        for #ident<#upstream_ty, #subscription_ty, Item, Error, #generic_params>
        where
            #upstream_ty: core::#upstream_ty<#subscription_ty, Item, Error> + Send,
            #subscription_ty: core::#subscription_ty + Send + Sync + 'static,
            Item: Send + 'static,
            Error: Send + 'static,
            #(#generic_params_iter: Send + 'static,)*
            #predicates
        {
            async fn subscribe<Downstream>(self, downstream: Downstream)
            where
                Downstream: core::#subscriber_trait<#downstream_params> + Send + 'static,
            {
                self.upstream.subscribe(#subscriber_ident::new(downstream, #(self.#field_idents),*)).await
            }
        }
    }
}

fn downstream_params(args: &Args) -> proc_macro2::TokenStream {
    let subscription = args.subscription();
    let item = args.item();
    let error = args.error();
    quote! {#subscription, #item, #error}
}

fn upstream_ty_from(args: &Args) -> Ident {
    match args.upstream {
        UpstreamTy::Flow => format_ident!("Flow"),
        UpstreamTy::Observable => format_ident!("Observable"),
    }
}

fn subscription_ty_from(args: &Args) -> Ident {
    match args.upstream {
        UpstreamTy::Flow => format_ident!("Subscription"),
        UpstreamTy::Observable => format_ident!("Cancellable"),
    }
}

fn subscriber_ident_from(args: &Args, ident: &Ident) -> syn::Type {
    let ident = format_ident!("{}{}", ident, args.subscriber_trait());
    parse_quote! {#ident}
}
