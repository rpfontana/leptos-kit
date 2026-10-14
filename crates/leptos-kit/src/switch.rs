use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[component(element = "button", skip(r#type, role))]
pub fn Switch(
    #[prop(into, default = "default".into())] size: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] checked: Option<Signal<bool>>,
    #[prop(optional)] default_checked: bool,
    #[prop(optional, into)] on_checked_change: Option<Callback<(bool, &'static str)>>,
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
        <primitives::SwitchRoot
            checked=checked
            default_checked=default_checked
            on_checked_change=on_checked_change
            disabled=disabled
            read_only=read_only
            required=required
            name=name
            form=form
            value=value
            unchecked_value=unchecked_value
            tabindex=tabindex
            class=classes("cn-switch peer group/switch", class)
            attr:data-slot="switch"
            attr:data-size=move || {
                #[cfg(debug_assertions)]
                check(size.get());
                size.get()
            }
            {..attrs}
        >
            <primitives::SwitchThumb class="cn-switch-thumb" attr:data-slot="switch-thumb" />
        </primitives::SwitchRoot>
    }
}

#[cfg(debug_assertions)]
fn check(size: &str) {
    const SIZES: &[&str] = &["default", "sm"];
    if !SIZES.contains(&size) {
        leptos::logging::warn!("Switch: unknown size `{size}`, expected one of {SIZES:?}");
    }
}
