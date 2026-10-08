use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "span")]
pub fn Badge(
    #[prop(default = "default")] variant: &'static str,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    #[cfg(debug_assertions)]
    check(variant);
    view! {
        <span
            class=classes(format!("cn-badge group/badge cn-badge-variant-{variant}"), class)
            data-variant=variant
            data-slot="badge"
            {..attrs}
        >
            {children.map(|children| children())}
        </span>
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
