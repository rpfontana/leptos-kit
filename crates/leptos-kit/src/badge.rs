use super::utils::classes;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

primitives::keywords! {
    #[derive(Default)]
    #[non_exhaustive]
    pub enum BadgeVariant {
        #[default]
        Default = "default",
        Secondary = "secondary",
        Destructive = "destructive",
        Outline = "outline",
        Ghost = "ghost",
        Link = "link",
    }
}

#[component(element = "span")]
pub fn Badge(
    #[prop(into, default = BadgeVariant::Default.into())] variant: Signal<BadgeVariant>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] href: Option<Signal<String>>,
    #[prop(optional, into)] target: Option<Signal<String>>,
    #[prop(optional, into)] rel: Option<Signal<String>>,
    #[prop(optional, into)] download: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        format!(
            "cn-badge group/badge cn-badge-variant-{}",
            variant.get().as_str()
        )
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
                data-variant=primitives::keyword(variant)
                data-slot="badge"
                {..attrs}
            >
                {children.map(|children| children())}
            </a>
        }),
        None => Either::Right(view! {
            <span class=class data-variant=primitives::keyword(variant) data-slot="badge" {..attrs}>
                {children.map(|children| children())}
            </span>
        }),
    }
}
