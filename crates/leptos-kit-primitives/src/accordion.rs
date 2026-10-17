use crate::button::button_attributes;
use crate::disclosure::Disclosure;
use crate::{Controlled, use_id};
use leptos::context::Provider;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[derive(Clone, Copy)]
struct AccordionContext {
    value: Controlled<Vec<String>>,
    multiple: Signal<bool>,
    disabled: Signal<bool>,
    orientation: Signal<&'static str>,
    hidden_until_found: Signal<bool>,
    items: StoredValue<usize>,
}

impl AccordionContext {
    fn change(&self, item: &str, open: bool, reason: &'static str) {
        let current = self.value.get_untracked();
        let next = if !self.multiple.get_untracked() {
            if current.first().is_some_and(|first| first == item) {
                Vec::new()
            } else {
                vec![item.to_string()]
            }
        } else if open {
            let mut next = current;
            next.push(item.to_string());
            next
        } else {
            current.into_iter().filter(|value| value != item).collect()
        };
        self.value.set(next, reason);
    }
}

#[derive(Clone, Copy)]
struct ItemContext {
    disclosure: Disclosure,
    index: usize,
    orientation: Signal<&'static str>,
    trigger_id: StoredValue<String>,
}

impl ItemContext {
    // what the item, its header and its panel show of the item's state
    fn attributes(self) -> impl leptos::attr::Attribute {
        let Self {
            disclosure,
            index,
            orientation,
            ..
        } = self;
        let state = view! {
            <{..}
                data-index=index.to_string()
                data-disabled=move || disclosure.disabled.get().then_some("")
                data-orientation=orientation
                data-hidden=move || disclosure.is_hidden().then_some("")
            />
        };
        (disclosure.open_attributes(), state)
    }
}

#[component(element = "div")]
pub fn AccordionRoot(
    #[prop(optional_no_strip, into)] value: Option<Signal<Vec<String>>>,
    #[prop(optional)] default_value: Vec<String>,
    #[prop(optional_no_strip, into)] on_value_change: Option<Callback<(Vec<String>, &'static str)>>,
    #[prop(optional, into)] multiple: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(into, default = "vertical".into())] orientation: Signal<&'static str>,
    #[prop(optional, into)] hidden_until_found: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = AccordionContext {
        value: Controlled::new(value, default_value, on_value_change),
        multiple,
        disabled,
        orientation,
        hidden_until_found,
        items: StoredValue::new(0),
    };
    view! {
        <div
            {..attrs}
            data-disabled=move || disabled.get().then_some("")
            data-orientation=orientation
        >
            <Provider value=context>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "div")]
pub fn AccordionItem(
    #[prop(optional_no_strip, into)] value: Option<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional_no_strip, into)] on_open_change: Option<Callback<(bool, &'static str)>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let root = expect_context::<AccordionContext>();
    let value = StoredValue::new(value.unwrap_or_else(use_id));
    let index = root
        .items
        .try_update_value(|items| {
            *items += 1;
            *items - 1
        })
        .unwrap_or_default();
    let open = Signal::derive(move || value.with_value(|value| root.value.get().contains(value)));
    let changed = Callback::new(move |(open, reason): (bool, &'static str)| {
        if let Some(on_open_change) = on_open_change {
            on_open_change.run((open, reason));
        }
        value.with_value(|value| root.change(value, open, reason));
    });
    let disabled = Signal::derive(move || disabled.get() || root.disabled.get());
    let disclosure = Disclosure::new(Controlled::new(Some(open), false, Some(changed)), disabled);
    let item = ItemContext {
        disclosure,
        index,
        orientation: root.orientation,
        trigger_id: StoredValue::new(use_id()),
    };
    view! {
        <div {..attrs} {..item.attributes()}>
            <Provider value=disclosure>
                <Provider value=item>{children.map(|children| children())}</Provider>
            </Provider>
        </div>
    }
}

#[component(element = "h3")]
pub fn AccordionHeader(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let item = expect_context::<ItemContext>();
    view! {
        <h3 {..attrs} {..item.attributes()}>
            {children.map(|children| children())}
        </h3>
    }
}

#[component(element = "button", skip(r#type, id))]
pub fn AccordionTrigger(
    #[prop(optional_no_strip, into)] id: Option<String>,
    #[prop(optional_no_strip, into)] disabled: Option<Signal<bool>>,
    #[prop(optional_no_strip, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let item = expect_context::<ItemContext>();
    let disclosure = item.disclosure;
    // the panel comes after its trigger, so it labels itself with the id set here
    if let Some(id) = id {
        item.trigger_id.set_value(id);
    }
    let disabled =
        move || disabled.is_some_and(|disabled| disabled.get()) || disclosure.disabled.get();
    let open = move || disclosure.is_open();
    view! {
        <button
            {..attrs}
            {..button_attributes(disabled, || true, tabindex)}
            type="button"
            id=item.trigger_id.get_value()
            aria-controls=move || open().then(|| disclosure.panel_id.get())
            aria-expanded=move || open().to_string()
            data-panel-open=move || open().then_some("")
            data-orientation=item.orientation
            on:click=move |_| disclosure.toggle()
        >
            {children.map(|children| children())}
        </button>
    }
}

// always in the dom, hidden when closed, as the collapsible's
#[component(element = "div", skip(id, role))]
pub fn AccordionPanel(
    #[prop(optional_no_strip, into)] id: Option<String>,
    #[prop(optional_no_strip, into)] hidden_until_found: Option<Signal<bool>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let root = expect_context::<AccordionContext>();
    let item = expect_context::<ItemContext>();
    let disclosure = item.disclosure;
    let hidden_until_found = Signal::derive(move || {
        hidden_until_found.map_or_else(|| root.hidden_until_found.get(), |hidden| hidden.get())
    });
    view! {
        <div
            {..attrs}
            {..item.attributes()}
            {..disclosure.phase_attributes()}
            {..disclosure.panel_attributes(
                "--accordion-panel-height",
                "--accordion-panel-width",
                hidden_until_found,
            )}
            node_ref=disclosure.panel
            id=id.unwrap_or_else(|| disclosure.panel_id.get())
            role="region"
            aria-labelledby=item.trigger_id.get_value()
        >
            {children.map(|children| children())}
        </div>
    }
}
