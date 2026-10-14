use crate::form::checked_input;
use crate::{Controlled, reason};
use leptos::context::Provider;
use leptos::ev;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_kit_macros::component;
use web_sys::{HtmlButtonElement, HtmlElement, HtmlFormElement, HtmlInputElement};

#[derive(Clone, Copy)]
struct CheckboxContext {
    checked: Controlled<bool>,
    indeterminate: Signal<bool>,
    disabled: Signal<bool>,
    read_only: Signal<bool>,
    required: Signal<bool>,
}

// base ui's nativeButton, for the same reason as the switch
#[component(element = "button", skip(r#type, role))]
pub fn CheckboxRoot(
    #[prop(optional_no_strip, into)] checked: Option<Signal<bool>>,
    #[prop(optional)] default_checked: bool,
    #[prop(optional_no_strip, into)] on_checked_change: Option<Callback<(bool, &'static str)>>,
    #[prop(optional, into)] indeterminate: Signal<bool>,
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
    let context = CheckboxContext {
        checked,
        indeterminate,
        disabled,
        read_only,
        required,
    };

    view! {
        <button
            {..attrs}
            {..state_attributes(context)}
            type="button"
            role="checkbox"
            tabindex=move || tabindex.map_or(0, |tabindex| tabindex.get())
            disabled=move || disabled.get()
            aria-checked=move || {
                if indeterminate.get() { "mixed".to_string() } else { checked.get().to_string() }
            }
            aria-readonly=move || read_only.get().then_some("true")
            aria-required=move || required.get().then_some("true")
            on:click=move |_| {
                if !read_only.get_untracked() {
                    checked.set(!checked.get_untracked(), reason::NONE);
                }
            }
            on:keydown=move |event: ev::KeyboardEvent| {
                // like a native checkbox, enter submits the form rather than ticking it
                if event.key() != "Enter" || event.default_prevented() {
                    return;
                }
                event.prevent_default();
                let submitter = event_target::<HtmlButtonElement>(&event)
                    .form()
                    .and_then(|form| default_submitter(&form));
                if let Some(submitter) = submitter {
                    submitter.click();
                }
            }
        >
            <Provider value=context>{children.map(|children| children())}</Provider>
            {checked_input(checked, name, form, value, unchecked_value, disabled)}
        </button>
    }
}

// hidden rather than unmounted: leptos's dev builds drop a caller's attr: from an element that comes back
#[component(element = "span")]
pub fn CheckboxIndicator(
    #[prop(optional, into)] keep_mounted: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<CheckboxContext>();
    let shown = move || keep_mounted.get() || context.checked.get() || context.indeterminate.get();
    view! {
        <span {..attrs} {..state_attributes(context)} hidden=move || !shown()>
            {children.map(|children| children())}
        </span>
    }
}

fn state_attributes(context: CheckboxContext) -> impl leptos::attr::Attribute {
    let CheckboxContext {
        checked,
        indeterminate,
        disabled,
        read_only,
        required,
    } = context;
    let ticked =
        move |on: bool| move || (!indeterminate.get() && checked.get() == on).then_some("");
    view! {
        <{..}
            data-checked=ticked(true)
            data-unchecked=ticked(false)
            data-indeterminate=move || indeterminate.get().then_some("")
            data-disabled=move || disabled.get().then_some("")
            data-readonly=move || read_only.get().then_some("")
            data-required=move || required.get().then_some("")
        />
    }
}

// the button a browser clicks for an implicit submission
fn default_submitter(form: &HtmlFormElement) -> Option<HtmlElement> {
    let elements = form.elements();
    (0..elements.length())
        .filter_map(|index| elements.item(index))
        .find(|element| {
            let kind = match element.dyn_ref::<HtmlButtonElement>() {
                Some(button) => button.type_(),
                None => element
                    .dyn_ref::<HtmlInputElement>()
                    .map(HtmlInputElement::type_)
                    .unwrap_or_default(),
            };
            kind == "submit"
        })
        .and_then(|element| element.dyn_into::<HtmlElement>().ok())
}
