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
    upstream_subscription: Option<syn::LitStr>,
    #[darling(default)]
    upstream_item: Option<syn::LitStr>,
    #[darling(default)]
    upstream_error: Option<syn::LitStr>,
    #[darling(default)]
    subscription: Option<syn::LitStr>,
    #[darling(default)]
    item: Option<syn::LitStr>,
    #[darling(default)]
    error: Option<syn::LitStr>,
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

    fn type_or_else<F>(lit_str: &Option<syn::LitStr>, f: F) -> syn::Type
    where
        F: Fn() -> syn::Type,
    {
        lit_str
            .as_ref()
            .map(|s| match parse_str(&s.value()) {
                Ok(ty) => ty,
                Err(_) => abort!(s, "`#[operator]` expected a valid type"),
            })
            .unwrap_or_else(f)
    }

    fn upstream_params(&self) -> proc_macro2::TokenStream {
        let subscription =
            Self::type_or_else(&self.upstream_subscription, || self.subscription_ty());
        let item = Self::type_or_else(&self.upstream_item, || parse_quote! {Item});
        let error = Self::type_or_else(&self.upstream_error, || parse_quote! {Error});
        quote! {#subscription, #item, #error}
    }

    fn downstream_params(&self) -> proc_macro2::TokenStream {
        let subscription = Self::type_or_else(&self.subscription, || self.subscription_ty());
        let item = Self::type_or_else(&self.item, || parse_quote! {Item});
        let error = Self::type_or_else(&self.error, || parse_quote! {Error});
        quote! {#subscription, #item, #error}
    }

    fn upstream_ty(&self) -> syn::Type {
        match self.upstream {
            UpstreamTy::Flow => parse_quote! { Flow },
            UpstreamTy::Observable => parse_quote! { Observable },
        }
    }

    fn subscription_ty(&self) -> syn::Type {
        match self.upstream {
            UpstreamTy::Flow => parse_quote! { Subscription },
            UpstreamTy::Observable => parse_quote! { Cancellable },
        }
    }

    fn subscriber_ident(&self, ident: &Ident) -> syn::Type {
        let ident = format_ident!("{}{}", ident, self.subscriber_trait());
        parse_quote! {#ident}
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
    let upstream_ty = args.upstream_ty();
    let subscription_ty = args.subscription_ty();
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
    let upstream_params = args.upstream_params();
    let downstream_params = args.downstream_params();
    let subscriber_ident = args
        .subscriber()
        .unwrap_or_else(|| args.subscriber_ident(&ident));
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
            #upstream_ty: core::#upstream_ty<#upstream_params> + Send,
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
