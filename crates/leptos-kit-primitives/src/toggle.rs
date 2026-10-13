use crate::button::button_attributes;
use crate::{Controlled, reason};
use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "button", skip(r#type, form, value))]
pub fn Toggle(
    #[prop(optional, into)] pressed: Option<Signal<bool>>,
    #[prop(optional)] default_pressed: bool,
    #[prop(optional, into)] on_pressed_change: Option<Callback<(bool, &'static str)>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <button
            {..attrs}
            {..toggle_attributes(pressed, default_pressed, on_pressed_change, disabled, tabindex)}
        >
            {children.map(|children| children())}
        </button>
    }
}

pub fn toggle_attributes(
    pressed: Option<Signal<bool>>,
    default_pressed: bool,
    on_pressed_change: Option<Callback<(bool, &'static str)>>,
    disabled: Option<Signal<bool>>,
    tabindex: Option<Signal<i32>>,
) -> impl leptos::attr::Attribute {
    let pressed = Controlled::new(pressed, default_pressed, on_pressed_change);
    let disabled = move || disabled.is_some_and(|disabled| disabled.get());
    let attributes = view! {
        <{..}
            type="button"
            aria-pressed=move || pressed.get().to_string()
            data-pressed=move || pressed.get().then_some("")
            on:click=move |_| pressed.set(!pressed.get_untracked(), reason::NONE)
        />
    };
    (button_attributes(disabled, || false, tabindex), attributes)
}
