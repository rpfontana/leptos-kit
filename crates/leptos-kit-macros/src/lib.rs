mod html;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use syn::ext::IdentExt;
use syn::{FnArg, Ident, ItemFn, LitStr, Pat, parse_macro_input, parse_quote};

#[derive(Clone, Copy)]
enum Kind {
    Bool,
    Int,
    UInt,
    Float,
    Text,
}

struct Attribute {
    name: &'static str,
    kind: Kind,
    doc: &'static str,
}

/// `#[component]` plus an `Option<Signal<T>>` prop per attribute of `element`, gathered in `attrs`
#[proc_macro_attribute]
pub fn component(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut element: Option<LitStr> = None;
    let mut skip: Vec<Ident> = Vec::new();
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("element") {
            element = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("skip") {
            meta.parse_nested_meta(|meta| {
                skip.push(meta.path.require_ident()?.clone());
                Ok(())
            })
        } else {
            Err(meta.error("expected `element = \"…\"` or `skip(…)`"))
        }
    });
    parse_macro_input!(args with parser);
    let item = parse_macro_input!(item as ItemFn);

    let Some(element) = element else {
        return syn::Error::new(Span::call_site(), "missing `element = \"…\"`")
            .into_compile_error()
            .into();
    };
    expand(&element, &skip, item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

// a flat tuple past 26 attributes panics at runtime, a nested one counts as one
const GROUP: usize = 16;

fn expand(element: &LitStr, skip: &[Ident], mut item: ItemFn) -> syn::Result<TokenStream2> {
    let tag = element.value();
    let own = html::ELEMENTS
        .binary_search_by(|(name, _)| name.cmp(&tag.as_str()))
        .map(|i| html::ELEMENTS[i].1)
        .map_err(|_| syn::Error::new(element.span(), format!("`{tag}` isn't an HTML element")))?;
    // `input` has a `title` of its own, with a more specific description
    let attributes: Vec<&Attribute> = html::GLOBAL
        .iter()
        .filter(|global| !own.iter().any(|a| a.name == global.name))
        .chain(own)
        .collect();

    for ident in skip {
        let name = ident.unraw().to_string();
        if !attributes.iter().any(|a| a.name == name) {
            return Err(syn::Error::new(
                ident.span(),
                format!("`<{tag}>` has no `{name}` attribute to skip"),
            ));
        }
    }

    let declared: Vec<String> = item
        .sig
        .inputs
        .iter()
        .filter_map(|input| match input {
            FnArg::Typed(typed) => match &*typed.pat {
                Pat::Ident(pat) => Some(pat.ident.unraw().to_string()),
                _ => None,
            },
            FnArg::Receiver(_) => None,
        })
        .collect();

    let mut bundled = Vec::new();
    for attribute in attributes {
        if declared.iter().any(|name| name == attribute.name)
            || skip.iter().any(|ident| ident.unraw() == attribute.name)
        {
            continue;
        }

        let ident = ident(attribute.name);
        let ty = match attribute.kind {
            Kind::Bool => quote!(bool),
            Kind::Int => quote!(i32),
            Kind::UInt => quote!(u32),
            Kind::Float => quote!(f64),
            Kind::Text => quote!(::std::string::String),
        };
        let doc = attribute.doc;
        item.sig.inputs.push(parse_quote! {
            #[doc = #doc]
            #[prop(optional, into)]
            #ident: Option<::leptos::prelude::Signal<#ty>>
        });

        let name = attribute.name;
        // class and style use leptos's own attrs to merge with class:/style:, Either keeps None out
        bundled.push(match name {
            "class" | "style" => {
                let function = Ident::new(name, Span::call_site());
                quote! {
                    match #ident {
                        Some(value) => ::leptos::either::Either::Left(
                            ::leptos::tachys::html::#function::#function(value),
                        ),
                        None => ::leptos::either::Either::Right(()),
                    }
                }
            }
            _ => quote!(::leptos::attr::custom::custom_attribute(#name, #ident)),
        });
    }

    let groups = bundled.chunks(GROUP).map(|group| quote!((#(#group,)*)));
    let attrs = Ident::new("attrs", Span::call_site());
    item.block
        .stmts
        .insert(0, parse_quote!(let #attrs = (#(#groups,)*);));

    Ok(quote! {
        #[::leptos::component]
        #item
    })
}

/// `type`, `for`, `async`, `loop` and `as` are keywords
fn ident(name: &str) -> TokenStream2 {
    match syn::parse_str::<Ident>(name) {
        Ok(ident) => ident.into_token_stream(),
        Err(_) => Ident::new_raw(name, Span::call_site()).into_token_stream(),
    }
}
