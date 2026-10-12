use crate::reason;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_kit_macros::component;
use web_sys::HtmlInputElement;

#[component(element = "input")]
pub fn Input(
    #[prop(optional, into)] value: Option<Signal<String>>,
    #[prop(optional, into)] default_value: Option<String>,
    #[prop(optional, into)] on_value_change: Option<Callback<(String, &'static str)>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
) -> impl IntoView {
    view! { <input {..attrs} {..input_attributes(value, default_value, on_value_change, disabled)} /> }
}

pub fn input_attributes(
    value: Option<Signal<String>>,
    default_value: Option<String>,
    on_value_change: Option<Callback<(String, &'static str)>>,
    disabled: Option<Signal<bool>>,
) -> impl leptos::attr::Attribute {
    let disabled = move || disabled.is_some_and(|disabled| disabled.get());
    // a property never reaches the server's html, so the attribute carries the first value
    let initial = value.map(|value| value.get_untracked()).or(default_value);
    let live = match value {
        Some(value) => Either::Left(leptos::tachys::html::property::prop("value", value)),
        None => Either::Right(()),
    };
    let attributes = view! {
        <{..}
            value=initial
            disabled=disabled
            data-disabled=move || disabled().then_some("")
            on:input=move |event| {
                let input = event_target::<HtmlInputElement>(&event);
                if let Some(on_value_change) = on_value_change {
                    on_value_change.run((input.value(), reason::NONE));
                }
                // like react, a controlled input goes back to its value unless the caller took the new one
                if let Some(value) = value {
                    let value = value.get_untracked();
                    if input.value() != value {
                        input.set_value(&value);
                    }
                }
            }
        />
    };
    (attributes, live)
}
