use crate::form::checked_input;
use crate::radio_group::RadioGroupContext;
use crate::{ChangeReason, Controlled};
use leptos::context::Provider;
use leptos::ev;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[derive(Clone, Copy)]
struct RadioContext {
    checked: Signal<bool>,
    disabled: Signal<bool>,
    read_only: Signal<bool>,
    required: Signal<bool>,
}

// base ui's nativeButton, for the same reason as the switch
#[component(element = "button", skip(r#type, role, name, form, tabindex))]
pub fn RadioRoot(
    #[prop(into)] value: String,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] read_only: Signal<bool>,
    #[prop(optional, into)] required: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let group = expect_context::<RadioGroupContext>();
    let value = StoredValue::new(value);
    let checked = Signal::derive(move || {
        value.with_value(|value| group.value.with(|checked| checked.as_ref() == Some(value)))
    });
    let context = RadioContext {
        checked,
        disabled: Signal::derive(move || disabled.get() || group.disabled.get()),
        read_only: Signal::derive(move || read_only.get() || group.read_only.get()),
        required: Signal::derive(move || required.get() || group.required.get()),
    };
    let element = NodeRef::new();
    let id = group.composite.item(context.disabled, checked, element);
    // safari doesn't focus a clicked button, nor does a label, so the tab stop follows the pick
    let select = move || {
        if !context.disabled.get_untracked() && !context.read_only.get_untracked() {
            group.composite.highlight(id);
            group.value.set(Some(value.get_value()), ChangeReason::None);
        }
    };

    view! {
        <button
            {..attrs}
            {..state_attributes(context)}
            {..group.composite.item_attributes(id)}
            node_ref=element
            type="button"
            role="radio"
            tabindex=group.composite.tabindex(id)
            disabled=move || context.disabled.get()
            aria-checked=move || checked.get().to_string()
            on:click=move |_| select()
            on:focus=move |_| {
                if group.arrowing.get_value() {
                    select();
                }
            }
            on:keydown=move |event: ev::KeyboardEvent| {
                // only space picks a radio
                if event.key() == "Enter" {
                    event.prevent_default();
                }
            }
        >
            <Provider value=context>{children.map(|children| children())}</Provider>
            {checked_input(
                Controlled::new(Some(checked), false, None),
                group.name,
                group.form,
                Some(Signal::derive(move || value.get_value())),
                None,
                context.disabled,
            )}
        </button>
    }
}

// hidden rather than unmounted: leptos's dev builds drop a caller's attr: from an element that comes back
#[component(element = "span")]
pub fn RadioIndicator(
    #[prop(optional, into)] keep_mounted: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<RadioContext>();
    view! {
        <span
            {..attrs}
            {..state_attributes(context)}
            hidden=move || !keep_mounted.get() && !context.checked.get()
        >
            {children.map(|children| children())}
        </span>
    }
}

fn state_attributes(context: RadioContext) -> impl leptos::attr::Attribute {
    let RadioContext {
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
