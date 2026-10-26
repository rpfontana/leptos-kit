use super::button::{ButtonSize, ButtonVariant, button_classes};
use super::utils::classes;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

primitives::keywords! {
    #[derive(Default)]
    #[non_exhaustive]
    pub enum AttachmentState {
        #[default]
        Done = "done",
        Idle = "idle",
        Uploading = "uploading",
        Processing = "processing",
        Error = "error",
    }
}

primitives::keywords! {
    #[derive(Default)]
    #[non_exhaustive]
    pub enum AttachmentSize {
        #[default]
        Default = "default",
        Sm = "sm",
        Xs = "xs",
    }
}

primitives::keywords! {
    #[derive(Default)]
    #[non_exhaustive]
    pub enum AttachmentMediaVariant {
        #[default]
        Icon = "icon",
        Image = "image",
    }
}

#[component(element = "div")]
pub fn Attachment(
    #[prop(into, default = AttachmentState::Done.into())] state: Signal<AttachmentState>,
    #[prop(into, default = AttachmentSize::Default.into())] size: Signal<AttachmentSize>,
    #[prop(into, default = primitives::Orientation::Horizontal.into())] orientation: Signal<
        primitives::Orientation,
    >,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        format!(
            "cn-attachment group/attachment cn-attachment-size-{} cn-attachment-orientation-{}",
            size.get().as_str(),
            orientation.get().as_str()
        )
    };
    view! {
        <div
            class=classes(own, class)
            data-state=primitives::keyword(state)
            data-size=primitives::keyword(size)
            data-orientation=primitives::keyword(orientation)
            data-slot="attachment"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn AttachmentMedia(
    #[prop(into, default = AttachmentMediaVariant::Icon.into())] variant: Signal<
        AttachmentMediaVariant,
    >,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        format!(
            "cn-attachment-media cn-attachment-media-variant-{}",
            variant.get().as_str()
        )
    };
    view! {
        <div
            class=classes(own, class)
            data-variant=primitives::keyword(variant)
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
    #[prop(into, default = ButtonVariant::Ghost.into())] variant: Signal<ButtonVariant>,
    #[prop(into, default = ButtonSize::IconXs.into())] size: Signal<ButtonSize>,
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
