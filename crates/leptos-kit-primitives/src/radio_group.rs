use crate::Controlled;
use crate::composite::Composite;
use leptos::context::Provider;
use leptos::ev;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[derive(Clone, Copy)]
pub(crate) struct RadioGroupContext {
    pub(crate) value: Controlled<Option<String>>,
    pub(crate) disabled: Signal<bool>,
    pub(crate) read_only: Signal<bool>,
    pub(crate) required: Signal<bool>,
    pub(crate) name: Option<Signal<String>>,
    pub(crate) form: Option<Signal<String>>,
    pub(crate) composite: Composite,
    // set while an arrow key moves the focus, which then picks the radio it lands on
    pub(crate) arrowing: StoredValue<bool>,
}

#[component(element = "div", skip(role))]
pub fn RadioGroup(
    #[prop(optional_no_strip, into)] value: Option<Signal<Option<String>>>,
    #[prop(optional_no_strip, into)] default_value: Option<String>,
    #[prop(optional_no_strip, into)] on_value_change: Option<Callback<(String, &'static str)>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] read_only: Signal<bool>,
    #[prop(optional, into)] required: Signal<bool>,
    #[prop(optional_no_strip, into)] name: Option<Signal<String>>,
    #[prop(optional_no_strip, into)] form: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let on_change = on_value_change.map(|on_value_change| {
        Callback::new(move |(value, reason): (Option<String>, &'static str)| {
            if let Some(value) = value {
                on_value_change.run((value, reason));
            }
        })
    });
    let composite = Composite::new("both".into(), true.into()).allow_shift();
    let context = RadioGroupContext {
        value: Controlled::new(value, default_value, on_change),
        disabled,
        read_only,
        required,
        name,
        form,
        composite,
        arrowing: StoredValue::new(false),
    };
    let flag = |on: Signal<bool>| move || on.get().then_some("true");
    let data = |on: Signal<bool>| move || on.get().then_some("");
    view! {
        <div
            {..attrs}
            role="radiogroup"
            aria-required=flag(required)
            aria-disabled=flag(disabled)
            aria-readonly=flag(read_only)
            data-disabled=data(disabled)
            data-readonly=data(read_only)
            data-required=data(required)
            on:keydown=move |event: ev::KeyboardEvent| {
                context.arrowing.set_value(true);
                composite.navigate(&event);
                context.arrowing.set_value(false);
            }
        >
            <Provider value=context>{children.map(|children| children())}</Provider>
        </div>
    }
}
