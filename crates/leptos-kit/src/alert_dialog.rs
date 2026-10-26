use super::button::{ButtonSize, ButtonVariant, button_classes, button_look};
use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
#[non_exhaustive]
pub enum AlertDialogSize {
    #[default]
    Default,
    Sm,
}

pub use primitives::AlertDialogRoot as AlertDialog;

#[component(element = "button", skip(r#type))]
pub fn AlertDialogTrigger(
    #[prop(optional, into)] variant: Option<Signal<ButtonVariant>>,
    #[prop(optional, into)] size: Option<Signal<ButtonSize>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::DialogTrigger
            disabled=disabled.unwrap_or_default()
            tabindex=tabindex.unwrap_or_else(|| Signal::stored(0))
            attr:data-slot="alert-dialog-trigger"
            {..button_look(variant, size, class)}
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::DialogTrigger>
    }
}

#[component(element = "dialog", skip(id, open, closedby, style, role))]
pub fn AlertDialogPortal(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <primitives::DialogPortal attr:data-slot="alert-dialog-portal" {..attrs}>
            {children.map(|children| children())}
        </primitives::DialogPortal>
    }
}

#[component(element = "div", skip(role))]
pub fn AlertDialogOverlay(#[prop(optional, into)] class: Option<Signal<String>>) -> impl IntoView {
    view! {
        <primitives::DialogBackdrop
            class=classes("cn-alert-dialog-overlay", class)
            attr:data-slot="alert-dialog-overlay"
            {..attrs}
        />
    }
}

#[component(element = "div")]
pub fn AlertDialogContent(
    #[prop(into, default = AlertDialogSize::Default.into())] size: Signal<AlertDialogSize>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <AlertDialogPortal>
            <AlertDialogOverlay />
            <primitives::DialogPopup
                class=classes("cn-alert-dialog-content group/alert-dialog-content", class)
                attr:data-slot="alert-dialog-content"
                attr:data-size=primitives::keyword(size)
                {..attrs}
            >
                {children.map(|children| children())}
            </primitives::DialogPopup>
        </AlertDialogPortal>
    }
}

#[component(element = "div")]
pub fn AlertDialogHeader(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-alert-dialog-header", class) data-slot="alert-dialog-header" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn AlertDialogFooter(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-alert-dialog-footer", class) data-slot="alert-dialog-footer" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn AlertDialogMedia(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-alert-dialog-media", class) data-slot="alert-dialog-media" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

// the primitive makes up the id the dialog points at
#[component(element = "h2", skip(id))]
pub fn AlertDialogTitle(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::DialogTitle
            class=classes("cn-alert-dialog-title cn-font-heading", class)
            attr:data-slot="alert-dialog-title"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::DialogTitle>
    }
}

#[component(element = "p", skip(id))]
pub fn AlertDialogDescription(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::DialogDescription
            class=classes("cn-alert-dialog-description", class)
            attr:data-slot="alert-dialog-description"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::DialogDescription>
    }
}

// shadcn's is a plain Button, so it doesn't close the dialog by itself
#[component(element = "button")]
pub fn AlertDialogAction(
    #[prop(into, default = ButtonVariant::Default.into())] variant: Signal<ButtonVariant>,
    #[prop(into, default = ButtonSize::Default.into())] size: Signal<ButtonSize>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] r#type: Option<Signal<String>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        format!(
            "cn-alert-dialog-action {}",
            button_classes(variant.get(), size.get())
        )
    };
    let disabled = move || disabled.is_some_and(|disabled| disabled.get());
    view! {
        <button
            {..attrs}
            {..primitives::button_attributes(disabled, || false, tabindex)}
            class=classes(own, class)
            type=move || r#type.map_or_else(|| "button".to_string(), |r#type| r#type.get())
            data-slot="alert-dialog-action"
        >
            {children.map(|children| children())}
        </button>
    }
}

#[component(element = "button", skip(r#type))]
pub fn AlertDialogCancel(
    #[prop(into, default = ButtonVariant::Outline.into())] variant: Signal<ButtonVariant>,
    #[prop(into, default = ButtonSize::Default.into())] size: Signal<ButtonSize>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        format!(
            "cn-alert-dialog-cancel {}",
            button_classes(variant.get(), size.get())
        )
    };
    view! {
        <primitives::DialogClose
            disabled=disabled.unwrap_or_default()
            tabindex=tabindex.unwrap_or_else(|| Signal::stored(0))
            class=classes(own, class)
            attr:data-slot="alert-dialog-cancel"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::DialogClose>
    }
}
