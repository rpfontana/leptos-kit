use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "div", skip(role))]
pub fn ItemGroup(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div
            class=classes("cn-item-group group/item-group", class)
            role="list"
            data-slot="item-group"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div", skip(role))]
pub fn ItemSeparator(
    #[prop(into, default = "horizontal".into())] orientation: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
) -> impl IntoView {
    let orientation = move || {
        #[cfg(debug_assertions)]
        check(
            "ItemSeparator",
            "orientation",
            orientation.get(),
            ORIENTATIONS,
        );
        orientation.get()
    };
    view! {
        <div
            class=classes("cn-item-separator", class)
            role="separator"
            aria-orientation=orientation
            data-orientation=orientation
            data-slot="item-separator"
            {..attrs}
        />
    }
}

#[component(element = "div")]
pub fn Item(
    #[prop(into, default = "default".into())] variant: Signal<&'static str>,
    #[prop(into, default = "default".into())] size: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        #[cfg(debug_assertions)]
        {
            check("Item", "variant", variant.get(), VARIANTS);
            check("Item", "size", size.get(), SIZES);
        }
        format!(
            "cn-item group/item cn-item-variant-{} cn-item-size-{}",
            variant.get(),
            size.get()
        )
    };
    view! {
        <div
            class=classes(own, class)
            data-variant=variant
            data-size=size
            data-slot="item"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn ItemMedia(
    #[prop(into, default = "default".into())] variant: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        #[cfg(debug_assertions)]
        check("ItemMedia", "variant", variant.get(), MEDIA_VARIANTS);
        format!("cn-item-media cn-item-media-variant-{}", variant.get())
    };
    view! {
        <div class=classes(own, class) data-variant=variant data-slot="item-media" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn ItemContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-item-content", class) data-slot="item-content" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn ItemTitle(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-item-title", class) data-slot="item-title" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "p")]
pub fn ItemDescription(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <p class=classes("cn-item-description", class) data-slot="item-description" {..attrs}>
            {children.map(|children| children())}
        </p>
    }
}

#[component(element = "div")]
pub fn ItemActions(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-item-actions", class) data-slot="item-actions" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn ItemHeader(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-item-header", class) data-slot="item-header" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn ItemFooter(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-item-footer", class) data-slot="item-footer" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[cfg(debug_assertions)]
const VARIANTS: &[&str] = &["default", "outline", "muted"];
#[cfg(debug_assertions)]
const SIZES: &[&str] = &["default", "sm", "xs"];
#[cfg(debug_assertions)]
const MEDIA_VARIANTS: &[&str] = &["default", "icon", "image"];
#[cfg(debug_assertions)]
const ORIENTATIONS: &[&str] = &["horizontal", "vertical"];

#[cfg(debug_assertions)]
fn check(part: &str, prop: &str, value: &str, known: &[&str]) {
    if !known.contains(&value) {
        leptos::logging::warn!("{part}: unknown {prop} `{value}`, expected one of {known:?}");
    }
}
