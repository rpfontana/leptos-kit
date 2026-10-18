use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[component(element = "div", skip(role))]
pub fn RadioGroup(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] value: Option<Signal<Option<String>>>,
    #[prop(optional, into)] default_value: Option<String>,
    #[prop(optional, into)] on_value_change: Option<Callback<(String, &'static str)>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] read_only: Signal<bool>,
    #[prop(optional, into)] required: Signal<bool>,
    #[prop(optional, into)] name: Option<Signal<String>>,
    #[prop(optional, into)] form: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::RadioGroup
            value=value
            default_value=default_value
            on_value_change=on_value_change
            disabled=disabled
            read_only=read_only
            required=required
            name=name
            form=form
            class=classes("cn-radio-group", class)
            attr:data-slot="radio-group"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::RadioGroup>
    }
}

#[component(element = "button", skip(r#type, role, name, form, tabindex))]
pub fn RadioGroupItem(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into)] value: String,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] read_only: Signal<bool>,
    #[prop(optional, into)] required: Signal<bool>,
) -> impl IntoView {
    view! {
        <primitives::RadioRoot
            value=value
            disabled=disabled
            read_only=read_only
            required=required
            class=classes("cn-radio-group-item group/radio-group-item peer", class)
            attr:data-slot="radio-group-item"
            {..attrs}
        >
            <primitives::RadioIndicator
                class="cn-radio-group-indicator"
                attr:data-slot="radio-group-indicator"
            >
                <span class="cn-radio-group-indicator-icon" />
            </primitives::RadioIndicator>
        </primitives::RadioRoot>
    }
}
