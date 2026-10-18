use crate::button::button_attributes;
use crate::toggle_group::ToggleGroupContext;
use crate::{Controlled, reason, use_id};
use leptos::either::Either;
use leptos::prelude::*;
use leptos::tachys::html::node_ref::node_ref;
use leptos_kit_macros::component;

#[component(element = "button", skip(r#type, form, value))]
pub fn Toggle(
    #[prop(optional, into)] pressed: Option<Signal<bool>>,
    #[prop(optional)] default_pressed: bool,
    #[prop(optional, into)] on_pressed_change: Option<Callback<(bool, &'static str)>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional, into)] value: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <button
            {..attrs}
            {..toggle_attributes(pressed, default_pressed, on_pressed_change, disabled, tabindex, value)}
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
    value: Option<String>,
) -> impl leptos::attr::Attribute {
    let Some(group) = use_context::<ToggleGroupContext>() else {
        let pressed = Controlled::new(pressed, default_pressed, on_pressed_change);
        let disabled = move || disabled.is_some_and(|disabled| disabled.get());
        return Either::Left((
            button_attributes(disabled, || false, tabindex),
            pressed_attributes(pressed),
        ));
    };

    // in a group, the group's value says which items are pressed
    let value = StoredValue::new(value.unwrap_or_else(use_id));
    let pressed =
        Signal::derive(move || value.with_value(|value| group.value.get().contains(value)));
    let changed = Callback::new(move |(pressed, reason): (bool, &'static str)| {
        if let Some(on_pressed_change) = on_pressed_change {
            on_pressed_change.run((pressed, reason));
        }
        value.with_value(|value| group.change(value, pressed, reason));
    });
    let disabled = Signal::derive(move || {
        disabled.is_some_and(|disabled| disabled.get()) || group.disabled.get()
    });
    let element = NodeRef::new();
    let id = group.composite.item(disabled, false.into(), element);
    Either::Right((
        button_attributes(
            move || disabled.get(),
            || false,
            Some(group.composite.tabindex(id)),
        ),
        pressed_attributes(Controlled::new(Some(pressed), false, Some(changed))),
        group.composite.item_attributes(id),
        // base ui's composite buttons always say whether they're disabled
        view! { <{..} aria-disabled=move || disabled.get().to_string() /> },
        node_ref(element),
    ))
}

fn pressed_attributes(pressed: Controlled<bool>) -> impl leptos::attr::Attribute {
    view! {
        <{..}
            type="button"
            aria-pressed=move || pressed.get().to_string()
            data-pressed=move || pressed.get().then_some("")
            on:click=move |_| pressed.set(!pressed.get_untracked(), reason::NONE)
        />
    }
}
