use crate::{Controlled, reason};
use leptos::context::Provider;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[derive(Clone, Copy)]
struct SwitchContext {
    checked: Controlled<bool>,
    disabled: Signal<bool>,
    read_only: Signal<bool>,
    required: Signal<bool>,
}

// base ui's nativeButton: its default span needs the keyboard and the label's name rebuilt in wasm
#[component(element = "button", skip(r#type, role))]
pub fn SwitchRoot(
    #[prop(optional_no_strip, into)] checked: Option<Signal<bool>>,
    #[prop(optional)] default_checked: bool,
    #[prop(optional_no_strip, into)] on_checked_change: Option<Callback<(bool, &'static str)>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] read_only: Signal<bool>,
    #[prop(optional, into)] required: Signal<bool>,
    #[prop(optional_no_strip, into)] name: Option<Signal<String>>,
    #[prop(optional_no_strip, into)] form: Option<Signal<String>>,
    #[prop(optional_no_strip, into)] value: Option<Signal<String>>,
    #[prop(optional_no_strip, into)] unchecked_value: Option<Signal<String>>,
    #[prop(optional_no_strip, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let checked = Controlled::new(checked, default_checked, on_checked_change);
    let context = SwitchContext {
        checked,
        disabled,
        read_only,
        required,
    };
    let submitted = move || {
        if checked.get() {
            Some(value.map_or_else(|| "on".to_string(), |value| value.get()))
        } else {
            unchecked_value.map(|value| value.get())
        }
    };

    view! {
        <button
            {..attrs}
            {..state_attributes(context)}
            type="button"
            role="switch"
            tabindex=move || tabindex.map_or(0, |tabindex| tabindex.get())
            disabled=move || disabled.get()
            aria-checked=move || checked.get().to_string()
            aria-readonly=move || read_only.get().then_some("true")
            aria-required=move || required.get().then_some("true")
            on:click=move |_| {
                if !read_only.get_untracked() {
                    checked.set(!checked.get_untracked(), reason::NONE);
                }
            }
        >
            <Provider value=context>{children.map(|children| children())}</Provider>
            // a hidden input isn't labelable, so a wrapping <label> stays valid html
            {name.map(|name| {
                view! {
                    <input
                        type="hidden"
                        name=move || name.get()
                        form=move || form.map(|form| form.get())
                        value=move || submitted().unwrap_or_default()
                        disabled=move || disabled.get() || submitted().is_none()
                    />
                }
            })}
        </button>
    }
}

#[component(element = "span")]
pub fn SwitchThumb(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<SwitchContext>();
    view! {
        <span {..attrs} {..state_attributes(context)}>
            {children.map(|children| children())}
        </span>
    }
}

fn state_attributes(context: SwitchContext) -> impl leptos::attr::Attribute {
    let SwitchContext {
        checked,
        disabled,
        read_only,
        required,
    } = context;
    view! {
        <{..}
            data-checked=move || checked.get().then_some("")
            data-unchecked=move || (!checked.get()).then_some("")
            data-disabled=move || disabled.get().then_some("")
            data-readonly=move || read_only.get().then_some("")
            data-required=move || required.get().then_some("")
        />
    }
}
