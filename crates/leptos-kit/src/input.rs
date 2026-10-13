use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[component(element = "input")]
pub fn Input(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] value: Option<Signal<String>>,
    #[prop(optional, into)] default_value: Option<String>,
    #[prop(optional, into)] on_value_change: Option<Callback<(String, &'static str)>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
) -> impl IntoView {
    view! {
        <input
            {..attrs}
            {..primitives::input_attributes(value, default_value, on_value_change, disabled)}
            class=classes(INPUT_CLASS, class)
            data-slot="input"
        />
    }
}

pub(crate) const INPUT_CLASS: &str = "cn-input";
