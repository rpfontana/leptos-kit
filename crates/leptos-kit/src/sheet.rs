use super::button::{ButtonSize, ButtonVariant, button_classes, button_look};
use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
pub enum SheetSide {
    #[default]
    Right,
    Top,
    Bottom,
    Left,
}

pub use primitives::DialogRoot as Sheet;

#[component(element = "button", skip(r#type))]
pub fn SheetTrigger(
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
            attr:data-slot="sheet-trigger"
            {..button_look(variant, size, class)}
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::DialogTrigger>
    }
}

#[component(element = "button", skip(r#type))]
pub fn SheetClose(
    #[prop(optional, into)] variant: Option<Signal<ButtonVariant>>,
    #[prop(optional, into)] size: Option<Signal<ButtonSize>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::DialogClose
            disabled=disabled.unwrap_or_default()
            tabindex=tabindex.unwrap_or_else(|| Signal::stored(0))
            attr:data-slot="sheet-close"
            {..button_look(variant, size, class)}
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::DialogClose>
    }
}

#[component(element = "dialog", skip(id, open, closedby, style, role))]
pub fn SheetPortal(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <primitives::DialogPortal attr:data-slot="sheet-portal" {..attrs}>
            {children.map(|children| children())}
        </primitives::DialogPortal>
    }
}

#[component(element = "div", skip(role))]
pub fn SheetOverlay(#[prop(optional, into)] class: Option<Signal<String>>) -> impl IntoView {
    view! {
        <primitives::DialogBackdrop
            class=classes("cn-sheet-overlay", class)
            attr:data-slot="sheet-overlay"
            {..attrs}
        />
    }
}

#[component(element = "div")]
pub fn SheetContent(
    #[prop(into, default = SheetSide::Right.into())] side: Signal<SheetSide>,
    #[prop(into, default = true.into())] show_close_button: Signal<bool>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <SheetPortal>
            <SheetOverlay />
            <primitives::DialogPopup
                class=classes("cn-sheet-content", class)
                attr:data-slot="sheet-content"
                attr:data-side=primitives::keyword(side)
                {..attrs}
            >
                {children.map(|children| children())}
                {move || show_close_button
                    .get()
                    .then(|| {
                        view! {
                            <primitives::DialogClose
                                class=format!("{} cn-sheet-close", button_classes(ButtonVariant::Ghost, ButtonSize::IconSm))
                                attr:data-slot="sheet-close"
                            >
                                <Icon icon=semantic::SHEET_X />
                                <span class="sr-only">"Close"</span>
                            </primitives::DialogClose>
                        }
                    })}
            </primitives::DialogPopup>
        </SheetPortal>
    }
}

#[component(element = "div")]
pub fn SheetHeader(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-sheet-header", class) data-slot="sheet-header" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn SheetFooter(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-sheet-footer", class) data-slot="sheet-footer" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

// the primitive makes up the id the dialog points at
#[component(element = "h2", skip(id))]
pub fn SheetTitle(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::DialogTitle
            class=classes("cn-sheet-title cn-font-heading", class)
            attr:data-slot="sheet-title"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::DialogTitle>
    }
}

#[component(element = "p", skip(id))]
pub fn SheetDescription(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::DialogDescription
            class=classes("cn-sheet-description", class)
            attr:data-slot="sheet-description"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::DialogDescription>
    }
}
