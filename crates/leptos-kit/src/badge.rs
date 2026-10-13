use super::utils::classes;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "span")]
pub fn Badge(
    #[prop(into, default = "default".into())] variant: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] href: Option<Signal<String>>,
    #[prop(optional, into)] target: Option<Signal<String>>,
    #[prop(optional, into)] rel: Option<Signal<String>>,
    #[prop(optional, into)] download: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        #[cfg(debug_assertions)]
        check(variant.get());
        format!("cn-badge group/badge cn-badge-variant-{}", variant.get())
    };
    let class = classes(own, class);
    match href {
        Some(href) => Either::Left(view! {
            <a
                class=class
                href=href
                target=target
                rel=rel
                download=download
                data-variant=variant
                data-slot="badge"
                {..attrs}
            >
                {children.map(|children| children())}
            </a>
        }),
        None => Either::Right(view! {
            <span class=class data-variant=variant data-slot="badge" {..attrs}>
                {children.map(|children| children())}
            </span>
        }),
    }
}

#[cfg(debug_assertions)]
fn check(variant: &str) {
    const VARIANTS: &[&str] = &[
        "default",
        "secondary",
        "destructive",
        "outline",
        "ghost",
        "link",
    ];
    if !VARIANTS.contains(&variant) {
        leptos::logging::warn!("Badge: unknown variant `{variant}`, expected one of {VARIANTS:?}");
    }
}
