use crate::Controlled;
use crate::button::button_attributes;
use crate::disclosure::Disclosure;
use leptos::context::Provider;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "div")]
pub fn CollapsibleRoot(
    #[prop(optional_no_strip, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional_no_strip, into)] on_open_change: Option<Callback<(bool, &'static str)>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let disclosure = Disclosure::new(
        Controlled::new(open, default_open, on_open_change),
        disabled,
    );
    view! {
        <div
            {..attrs}
            {..disclosure.open_attributes()}
            {..disclosure.phase_attributes()}
            data-disabled=move || disabled.get().then_some("")
        >
            <Provider value=disclosure>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "button", skip(r#type))]
pub fn CollapsibleTrigger(
    #[prop(optional_no_strip, into)] disabled: Option<Signal<bool>>,
    #[prop(optional_no_strip, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let disclosure = expect_context::<Disclosure>();
    let disabled =
        move || disabled.map_or_else(|| disclosure.disabled.get(), |disabled| disabled.get());
    let open = move || disclosure.is_open();
    view! {
        <button
            {..attrs}
            {..button_attributes(disabled, || true, tabindex)}
            {..disclosure.phase_attributes()}
            type="button"
            aria-controls=move || open().then(|| disclosure.panel_id.get())
            aria-expanded=move || open().to_string()
            data-panel-open=move || open().then_some("")
            on:click=move |_| disclosure.toggle()
        >
            {children.map(|children| children())}
        </button>
    }
}

// always in the dom, hidden when closed: leptos's dev builds drop a caller's attr: from an element that comes back
#[component(element = "div", skip(id))]
pub fn CollapsiblePanel(
    #[prop(optional_no_strip, into)] id: Option<String>,
    #[prop(optional, into)] hidden_until_found: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let disclosure = expect_context::<Disclosure>();
    view! {
        <div
            {..attrs}
            {..disclosure.open_attributes()}
            {..disclosure.phase_attributes()}
            {..disclosure.panel_attributes(
                "--collapsible-panel-height",
                "--collapsible-panel-width",
                hidden_until_found,
            )}
            node_ref=disclosure.panel
            id=id.unwrap_or_else(|| disclosure.panel_id.get())
            data-disabled=move || disclosure.disabled.get().then_some("")
        >
            {children.map(|children| children())}
        </div>
    }
}
