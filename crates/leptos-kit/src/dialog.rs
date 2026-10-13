use super::button::{button_classes, button_look};
use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

pub use primitives::DialogRoot as Dialog;

#[component(element = "button", skip(r#type))]
pub fn DialogTrigger(
    #[prop(optional, into)] variant: Option<Signal<&'static str>>,
    #[prop(optional, into)] size: Option<Signal<&'static str>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::DialogTrigger
            disabled=disabled.unwrap_or_default()
            tabindex=tabindex.unwrap_or_else(|| Signal::stored(0))
            attr:data-slot="dialog-trigger"
            {..button_look(variant, size, class)}
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::DialogTrigger>
    }
}

#[component(element = "dialog", skip(id, open, closedby, style))]
pub fn DialogPortal(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <primitives::DialogPortal attr:data-slot="dialog-portal" {..attrs}>
            {children.map(|children| children())}
        </primitives::DialogPortal>
    }
}

#[component(element = "div", skip(role))]
pub fn DialogOverlay(#[prop(optional, into)] class: Option<Signal<String>>) -> impl IntoView {
    view! {
        <primitives::DialogBackdrop
            class=classes("cn-dialog-overlay", class)
            attr:data-slot="dialog-overlay"
            {..attrs}
        />
    }
}

#[component(element = "div")]
pub fn DialogContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into, default = true.into())] show_close_button: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <DialogPortal>
            <DialogOverlay />
            <primitives::DialogPopup
                class=classes("cn-dialog-content", class)
                attr:data-slot="dialog-content"
                {..attrs}
            >
                {children.map(|children| children())}
                {move || show_close_button
                    .get()
                    .then(|| {
                        view! {
                            <primitives::DialogClose
                                class=format!("{} cn-dialog-close", button_classes("ghost", "icon-sm"))
                                attr:data-slot="dialog-close"
                            >
                                <Icon icon=semantic::DIALOG_X />
                                <span class="sr-only">"Close"</span>
                            </primitives::DialogClose>
                        }
                    })}
            </primitives::DialogPopup>
        </DialogPortal>
    }
}

#[component(element = "div")]
pub fn DialogHeader(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-dialog-header", class) data-slot="dialog-header" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn DialogFooter(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] show_close_button: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-dialog-footer", class) data-slot="dialog-footer" {..attrs}>
            {children.map(|children| children())}
            {move || show_close_button
                .get()
                .then(|| {
                    view! {
                        <primitives::DialogClose
                            class=button_classes("outline", "default")
                            attr:data-slot="button"
                        >
                            "Close"
                        </primitives::DialogClose>
                    }
                })}
        </div>
    }
}

// the primitive makes up the id the dialog points at
#[component(element = "h2", skip(id))]
pub fn DialogTitle(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::DialogTitle
            class=classes("cn-dialog-title cn-font-heading", class)
            attr:data-slot="dialog-title"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::DialogTitle>
    }
}

#[component(element = "p", skip(id))]
pub fn DialogDescription(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::DialogDescription
            class=classes("cn-dialog-description", class)
            attr:data-slot="dialog-description"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::DialogDescription>
    }
}

#[component(element = "button", skip(r#type))]
pub fn DialogClose(
    #[prop(optional, into)] variant: Option<Signal<&'static str>>,
    #[prop(optional, into)] size: Option<Signal<&'static str>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::DialogClose
            disabled=disabled.unwrap_or_default()
            tabindex=tabindex.unwrap_or_else(|| Signal::stored(0))
            attr:data-slot="dialog-close"
            {..button_look(variant, size, class)}
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::DialogClose>
    }
}
