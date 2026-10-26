use super::button::{Button, ButtonVariant};
use super::input::INPUT_CLASS;
use super::textarea::TEXTAREA_CLASS;
use super::utils::classes;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::{Element, HtmlElement};
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
pub enum InputGroupAddonAlign {
    #[default]
    InlineStart,
    InlineEnd,
    BlockStart,
    BlockEnd,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
#[non_exhaustive]
pub enum InputGroupButtonSize {
    #[default]
    Xs,
    Sm,
    IconXs,
    IconSm,
}

#[component(element = "div", skip(role))]
pub fn InputGroup(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div
            class=classes("group/input-group cn-input-group", class)
            role="group"
            data-slot="input-group"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div", skip(role))]
pub fn InputGroupAddon(
    #[prop(into, default = InputGroupAddonAlign::InlineStart.into())] align: Signal<
        InputGroupAddonAlign,
    >,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        format!(
            "cn-input-group-addon cn-input-group-addon-align-{}",
            align.get().as_str()
        )
    };
    // a click on the addon, unless it hit a button, focuses the group's input
    let focus_input = move |event: leptos::ev::MouseEvent| {
        if event_target::<Element>(&event)
            .closest("button")
            .ok()
            .flatten()
            .is_some()
        {
            return;
        }
        let input = event
            .current_target()
            .and_then(|addon| addon.unchecked_into::<Element>().parent_element())
            .and_then(|group| group.query_selector("input").ok().flatten());
        if let Some(input) = input {
            let _ = input.unchecked_into::<HtmlElement>().focus();
        }
    };
    view! {
        <div
            class=classes(own, class)
            role="group"
            data-align=primitives::keyword(align)
            data-slot="input-group-addon"
            {..attrs}
            on:click=focus_input
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "button")]
pub fn InputGroupButton(
    #[prop(into, default = ButtonVariant::Ghost.into())] variant: Signal<ButtonVariant>,
    #[prop(into, default = InputGroupButtonSize::Xs.into())] size: Signal<InputGroupButtonSize>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] focusable_when_disabled: Option<Signal<bool>>,
    #[prop(into, default = "button".into())] r#type: Signal<String>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        format!(
            "cn-input-group-button cn-input-group-button-size-{}",
            size.get().as_str()
        )
    };
    view! {
        <Button
            variant=variant
            class=Signal::derive(classes(own, class))
            disabled=disabled.unwrap_or_default()
            focusable_when_disabled=focusable_when_disabled.unwrap_or_default()
            r#type=r#type
            tabindex=tabindex.unwrap_or_else(|| Signal::stored(0))
            {..attrs}
            attr:data-size=primitives::keyword(size)
        >
            {children.map(|children| children())}
        </Button>
    }
}

#[component(element = "span")]
pub fn InputGroupText(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <span class=classes("cn-input-group-text", class) {..attrs}>
            {children.map(|children| children())}
        </span>
    }
}

#[component(element = "input")]
pub fn InputGroupInput(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] value: Option<Signal<String>>,
    #[prop(optional, into)] default_value: Option<String>,
    #[prop(optional, into)] on_value_change: Option<Callback<(String, primitives::ChangeReason)>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
) -> impl IntoView {
    view! {
        <input
            {..attrs}
            {..primitives::input_attributes(value, default_value, on_value_change, disabled)}
            class=classes(move || format!("{INPUT_CLASS} cn-input-group-input"), class)
            data-slot="input-group-control"
        />
    }
}

#[component(element = "textarea")]
pub fn InputGroupTextarea(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <textarea
            class=classes(move || format!("{TEXTAREA_CLASS} cn-input-group-textarea"), class)
            data-slot="input-group-control"
            {..attrs}
        >
            {children.map(|children| children())}
        </textarea>
    }
}
