use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "div")]
pub fn Empty(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-empty", class) data-slot="empty" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn EmptyHeader(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-empty-header", class) data-slot="empty-header" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn EmptyMedia(
    #[prop(default = "default")] variant: &'static str,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    #[cfg(debug_assertions)]
    check(variant);
    view! {
        <div
            class=classes(format!("cn-empty-media cn-empty-media-{variant}"), class)
            data-variant=variant
            data-slot="empty-icon"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn EmptyTitle(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-empty-title cn-font-heading", class) data-slot="empty-title" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn EmptyDescription(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-empty-description", class) data-slot="empty-description" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn EmptyContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-empty-content", class) data-slot="empty-content" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[cfg(debug_assertions)]
fn check(variant: &str) {
    const VARIANTS: &[&str] = &["default", "icon"];
    if !VARIANTS.contains(&variant) {
        leptos::logging::warn!(
            "EmptyMedia: unknown variant `{variant}`, expected one of {VARIANTS:?}"
        );
    }
}
