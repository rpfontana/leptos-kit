use super::button::{ButtonSize, ButtonVariant, button_look};
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[component(element = "div")]
pub fn Collapsible(
    #[prop(optional, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, primitives::ChangeReason)>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::CollapsibleRoot
            open=open
            default_open=default_open
            on_open_change=on_open_change
            disabled=disabled
            attr:data-slot="collapsible"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::CollapsibleRoot>
    }
}

#[component(element = "button", skip(r#type))]
pub fn CollapsibleTrigger(
    #[prop(optional, into)] variant: Option<Signal<ButtonVariant>>,
    #[prop(optional, into)] size: Option<Signal<ButtonSize>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::CollapsibleTrigger
            disabled=disabled
            tabindex=tabindex
            attr:data-slot="collapsible-trigger"
            {..button_look(variant, size, class)}
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::CollapsibleTrigger>
    }
}

#[component(element = "div", skip(id))]
pub fn CollapsibleContent(
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] hidden_until_found: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::CollapsiblePanel
            id=id
            hidden_until_found=hidden_until_found
            attr:data-slot="collapsible-content"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::CollapsiblePanel>
    }
}
