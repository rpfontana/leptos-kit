use leptos::ev;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "button")]
pub fn Button(
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] focusable_when_disabled: Option<Signal<bool>>,
    #[prop(optional, into)] r#type: Option<Signal<String>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let disabled = move || disabled.is_some_and(|disabled| disabled.get());
    let focusable_when_disabled =
        move || focusable_when_disabled.is_some_and(|focusable| focusable.get());
    view! {
        <button
            {..attrs}
            {..button_attributes(disabled, focusable_when_disabled, tabindex)}
            type=move || r#type.map_or_else(|| "button".to_string(), |r#type| r#type.get())
        >
            {children.map(|children| children())}
        </button>
    }
}

pub fn button_attributes(
    disabled: impl Fn() -> bool + Copy + Send + Sync + 'static,
    focusable_when_disabled: impl Fn() -> bool + Copy + Send + Sync + 'static,
    tabindex: Option<Signal<i32>>,
) -> impl leptos::attr::Attribute {
    let focusable_disabled = move || focusable_when_disabled() && disabled();
    let swallow_clicks = leptos::tachys::html::event::on(
        // not delegated, so it runs before the caller's on:click
        ev::Custom::<ev::MouseEvent>::new("click"),
        move |event: ev::MouseEvent| {
            if focusable_disabled() {
                event.prevent_default();
                event.stop_immediate_propagation();
            }
        },
    );
    let attributes = view! {
        <{..}
            tabindex=move || tabindex.map_or(0, |tabindex| tabindex.get())
            disabled=move || !focusable_when_disabled() && disabled()
            aria-disabled=move || focusable_when_disabled().then(|| disabled().to_string())
            data-disabled=move || disabled().then_some("")
            on:keydown=move |event: ev::KeyboardEvent| {
                if focusable_disabled() && event.key() != "Tab" {
                    event.prevent_default();
                }
            }
        />
    };
    (attributes, swallow_clicks)
}

// base ui's composite item on a native button: aria-disabled always said, and the button disabled too
pub(crate) fn composite_button_attributes(
    disabled: impl Fn() -> bool + Copy + Send + Sync + 'static,
    tabindex: Signal<i32>,
) -> impl leptos::attr::Attribute {
    view! {
        <{..}
            tabindex=move || tabindex.get()
            disabled=disabled
            aria-disabled=move || disabled().to_string()
            data-disabled=move || disabled().then_some("")
        />
    }
}
