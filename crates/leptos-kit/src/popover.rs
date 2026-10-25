use super::button::{ButtonSize, ButtonVariant, button_look};
use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

pub use primitives::PopoverRoot as Popover;

#[component(element = "button", skip(r#type, popovertarget, popovertargetaction))]
pub fn PopoverTrigger(
    #[prop(optional, into)] variant: Option<Signal<ButtonVariant>>,
    #[prop(optional, into)] size: Option<Signal<ButtonSize>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::PopoverTrigger
            disabled=disabled.unwrap_or_default()
            tabindex=tabindex.unwrap_or_else(|| Signal::stored(0))
            attr:data-slot="popover-trigger"
            {..button_look(variant, size, class)}
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::PopoverTrigger>
    }
}

#[component(element = "div", skip(id, role))]
pub fn PopoverContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into, default = primitives::Side::Bottom.into())] side: Signal<primitives::Side>,
    #[prop(into, default = primitives::Align::Center.into())] align: Signal<primitives::Align>,
    #[prop(into, default = 4.0.into())] side_offset: Signal<f64>,
    #[prop(optional, into)] align_offset: Signal<f64>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::PopoverPositioner side align side_offset align_offset class="isolate z-50">
            <primitives::PopoverPopup
                class=classes("cn-popover-content cn-popover-content-logical", class)
                attr:data-slot="popover-content"
                {..attrs}
            >
                {children.map(|children| children())}
            </primitives::PopoverPopup>
        </primitives::PopoverPositioner>
    }
}

#[component(element = "div")]
pub fn PopoverHeader(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-popover-header", class) data-slot="popover-header" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "h2", skip(id))]
pub fn PopoverTitle(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::PopoverTitle
            class=classes("cn-popover-title", class)
            attr:data-slot="popover-title"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::PopoverTitle>
    }
}

#[component(element = "p", skip(id))]
pub fn PopoverDescription(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::PopoverDescription
            class=classes("cn-popover-description", class)
            attr:data-slot="popover-description"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::PopoverDescription>
    }
}
