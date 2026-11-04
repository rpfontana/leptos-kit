use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident};

// the trait as the crate being built sees it: through the facade, the primitives, or from inside either
fn keyword_trait() -> TokenStream {
    let named = |name: &str| {
        let name = Ident::new(name, Span::call_site());
        quote!(::#name)
    };
    match crate_name("leptos-kit") {
        Ok(FoundCrate::Itself) => quote!(crate::primitives::Keyword),
        Ok(FoundCrate::Name(name)) => {
            let facade = named(&name);
            quote!(#facade::primitives::Keyword)
        }
        Err(_) => match crate_name("leptos-kit-primitives") {
            Ok(FoundCrate::Itself) => quote!(crate::Keyword),
            Ok(FoundCrate::Name(name)) => {
                let primitives = named(&name);
                quote!(#primitives::Keyword)
            }
            Err(_) => quote!(::leptos_kit::primitives::Keyword),
        },
    }
}

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "`Keyword` derives on enums only",
        ));
    };
    let name = &input.ident;
    let keyword = keyword_trait();
    let arms = data
        .variants
        .iter()
        .map(|variant| {
            if !matches!(variant.fields, Fields::Unit) {
                return Err(syn::Error::new_spanned(
                    variant,
                    "a keyword variant carries no data",
                ));
            }
            let ident = &variant.ident;
            let text = kebab(&ident.to_string());
            Ok(quote!(Self::#ident => #text))
        })
        .collect::<syn::Result<Vec<_>>>()?;
    Ok(quote! {
        impl #name {
            pub const fn as_str(self) -> &'static str {
                match self {
                    #(#arms,)*
                }
            }
        }

        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl #keyword for #name {
            fn as_str(self) -> &'static str {
                #name::as_str(self)
            }
        }
    })
}

// `InlineStart` to `inline-start`
fn kebab(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    for (index, char) in name.chars().enumerate() {
        if char.is_ascii_uppercase() && index > 0 {
            out.push('-');
        }
        out.push(char.to_ascii_lowercase());
    }
    out
}
