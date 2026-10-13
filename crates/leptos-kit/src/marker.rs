use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "div")]
pub fn Marker(
    #[prop(into, default = "default".into())] variant: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        #[cfg(debug_assertions)]
        check(variant.get());
        format!("cn-marker group/marker cn-marker-variant-{}", variant.get())
    };
    view! {
        <div class=classes(own, class) data-variant=variant data-slot="marker" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "span")]
pub fn MarkerIcon(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <span
            class=classes("cn-marker-icon", class)
            data-slot="marker-icon"
            aria-hidden="true"
            {..attrs}
        >
            {children.map(|children| children())}
        </span>
    }
}

#[component(element = "span")]
pub fn MarkerContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <span class=classes("cn-marker-content", class) data-slot="marker-content" {..attrs}>
            {children.map(|children| children())}
        </span>
    }
}

#[cfg(debug_assertions)]
fn check(variant: &str) {
    const VARIANTS: &[&str] = &["default", "separator", "border"];
    if !VARIANTS.contains(&variant) {
        leptos::logging::warn!("Marker: unknown variant `{variant}`, expected one of {VARIANTS:?}");
    }
}
