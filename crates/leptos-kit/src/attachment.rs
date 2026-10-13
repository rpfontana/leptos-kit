use super::button::button_classes;
use super::utils::classes;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[component(element = "div")]
pub fn Attachment(
    #[prop(into, default = "done".into())] state: Signal<&'static str>,
    #[prop(into, default = "default".into())] size: Signal<&'static str>,
    #[prop(into, default = "horizontal".into())] orientation: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        #[cfg(debug_assertions)]
        {
            check("Attachment", "state", state.get(), STATES);
            check("Attachment", "size", size.get(), SIZES);
            check("Attachment", "orientation", orientation.get(), ORIENTATIONS);
        }
        format!(
            "cn-attachment group/attachment cn-attachment-size-{} cn-attachment-orientation-{}",
            size.get(),
            orientation.get()
        )
    };
    view! {
        <div
            class=classes(own, class)
            data-state=state
            data-size=size
            data-orientation=orientation
            data-slot="attachment"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn AttachmentMedia(
    #[prop(into, default = "icon".into())] variant: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        #[cfg(debug_assertions)]
        check("AttachmentMedia", "variant", variant.get(), MEDIA_VARIANTS);
        format!(
            "cn-attachment-media cn-attachment-media-variant-{}",
            variant.get()
        )
    };
    view! {
        <div
            class=classes(own, class)
            data-variant=variant
            data-slot="attachment-media"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn AttachmentContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-attachment-content", class) data-slot="attachment-content" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "span")]
pub fn AttachmentTitle(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <span class=classes("cn-attachment-title", class) data-slot="attachment-title" {..attrs}>
            {children.map(|children| children())}
        </span>
    }
}

#[component(element = "span")]
pub fn AttachmentDescription(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <span
            class=classes("cn-attachment-description", class)
            data-slot="attachment-description"
            {..attrs}
        >
            {children.map(|children| children())}
        </span>
    }
}

#[component(element = "div")]
pub fn AttachmentActions(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-attachment-actions", class) data-slot="attachment-actions" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "button")]
pub fn AttachmentAction(
    #[prop(into, default = "ghost".into())] variant: Signal<&'static str>,
    #[prop(into, default = "icon-xs".into())] size: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] focusable_when_disabled: Option<Signal<bool>>,
    #[prop(optional, into)] r#type: Option<Signal<String>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let disabled = move || disabled.is_some_and(|disabled| disabled.get());
    let focusable_when_disabled =
        move || focusable_when_disabled.is_some_and(|focusable| focusable.get());
    let own = move || {
        format!(
            "{} cn-attachment-action",
            button_classes(variant.get(), size.get())
        )
    };
    view! {
        <button
            {..attrs}
            {..primitives::button_attributes(disabled, focusable_when_disabled, tabindex)}
            class=classes(own, class)
            type=move || r#type.map_or_else(|| "button".to_string(), |r#type| r#type.get())
            data-slot="attachment-action"
        >
            {children.map(|children| children())}
        </button>
    }
}

#[component(element = "button")]
pub fn AttachmentTrigger(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] r#type: Option<Signal<String>>,
    #[prop(optional, into)] href: Option<Signal<String>>,
    #[prop(optional, into)] target: Option<Signal<String>>,
    #[prop(optional, into)] rel: Option<Signal<String>>,
    #[prop(optional, into)] download: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let class = classes("cn-attachment-trigger", class);
    match href {
        Some(href) => Either::Left(view! {
            <a
                {..attrs}
                class=class
                href=href
                target=target
                rel=rel
                download=download
                data-slot="attachment-trigger"
            >
                {children.map(|children| children())}
            </a>
        }),
        None => Either::Right(view! {
            <button
                {..attrs}
                class=class
                type=move || r#type.map_or_else(|| "button".to_string(), |r#type| r#type.get())
                data-slot="attachment-trigger"
            >
                {children.map(|children| children())}
            </button>
        }),
    }
}

#[component(element = "div")]
pub fn AttachmentGroup(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-attachment-group", class) data-slot="attachment-group" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[cfg(debug_assertions)]
const STATES: &[&str] = &["idle", "uploading", "processing", "error", "done"];
#[cfg(debug_assertions)]
const SIZES: &[&str] = &["default", "sm", "xs"];
#[cfg(debug_assertions)]
const ORIENTATIONS: &[&str] = &["horizontal", "vertical"];
#[cfg(debug_assertions)]
const MEDIA_VARIANTS: &[&str] = &["icon", "image"];

#[cfg(debug_assertions)]
fn check(part: &str, prop: &str, value: &str, known: &[&str]) {
    if !known.contains(&value) {
        leptos::logging::warn!("{part}: unknown {prop} `{value}`, expected one of {known:?}");
    }
}
