use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "div", skip(role))]
pub fn Alert(
    #[prop(into, default = "default".into())] variant: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        #[cfg(debug_assertions)]
        check(variant.get());
        format!("cn-alert group/alert cn-alert-variant-{}", variant.get())
    };
    view! {
        <div
            class=classes(own, class)
            data-slot="alert"
            role="alert"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn AlertTitle(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-alert-title", class) data-slot="alert-title" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn AlertDescription(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-alert-description", class) data-slot="alert-description" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn AlertAction(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-alert-action", class) data-slot="alert-action" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[cfg(debug_assertions)]
fn check(variant: &str) {
    const VARIANTS: &[&str] = &["default", "destructive"];
    if !VARIANTS.contains(&variant) {
        leptos::logging::warn!("Alert: unknown variant `{variant}`, expected one of {VARIANTS:?}");
    }
}
