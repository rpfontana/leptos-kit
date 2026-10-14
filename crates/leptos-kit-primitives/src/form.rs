use crate::Controlled;
use leptos::prelude::*;

// a hidden input isn't labelable, so a wrapping <label> stays valid html
pub(crate) fn checked_input(
    checked: Controlled<bool>,
    name: Option<Signal<String>>,
    form: Option<Signal<String>>,
    value: Option<Signal<String>>,
    unchecked_value: Option<Signal<String>>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let submitted = move || {
        if checked.get() {
            Some(value.map_or_else(|| "on".to_string(), |value| value.get()))
        } else {
            unchecked_value.map(|value| value.get())
        }
    };
    name.map(|name| {
        view! {
            <input
                type="hidden"
                name=move || name.get()
                form=move || form.map(|form| form.get())
                value=move || submitted().unwrap_or_default()
                disabled=move || disabled.get() || submitted().is_none()
            />
        }
    })
}
