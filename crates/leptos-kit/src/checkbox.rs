use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[component(element = "button", skip(r#type, role))]
pub fn Checkbox(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] checked: Option<Signal<bool>>,
    #[prop(optional)] default_checked: bool,
    #[prop(optional, into)] on_checked_change: Option<Callback<(bool, primitives::ChangeReason)>>,
    #[prop(optional, into)] indeterminate: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] read_only: Signal<bool>,
    #[prop(optional, into)] required: Signal<bool>,
    #[prop(optional, into)] name: Option<Signal<String>>,
    #[prop(optional, into)] form: Option<Signal<String>>,
    #[prop(optional, into)] value: Option<Signal<String>>,
    #[prop(optional, into)] unchecked_value: Option<Signal<String>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
) -> impl IntoView {
    view! {
        <primitives::CheckboxRoot
            checked=checked
            default_checked=default_checked
            on_checked_change=on_checked_change
            indeterminate=indeterminate
            disabled=disabled
            read_only=read_only
            required=required
            name=name
            form=form
            value=value
            unchecked_value=unchecked_value
            tabindex=tabindex
            class=classes("cn-checkbox peer", class)
            attr:data-slot="checkbox"
            {..attrs}
        >
            <primitives::CheckboxIndicator
                class="cn-checkbox-indicator"
                attr:data-slot="checkbox-indicator"
            >
                <Icon icon=semantic::CHECKBOX_CHECK />
            </primitives::CheckboxIndicator>
        </primitives::CheckboxRoot>
    }
}
