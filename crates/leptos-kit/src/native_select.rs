use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};
use leptos_kit_macros::component;

#[component(element = "select")]
pub fn NativeSelect(
    #[prop(into, default = "default".into())] size: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let size = move || {
        #[cfg(debug_assertions)]
        check(size.get());
        size.get()
    };
    view! {
        <div
            class=classes("cn-native-select-wrapper group/native-select", class)
            data-slot="native-select-wrapper"
            data-size=size
        >
            <select class="cn-native-select" data-size=size data-slot="native-select" {..attrs}>
                {children.map(|children| children())}
            </select>
            <Icon
                icon=semantic::NATIVE_SELECT_CHEVRON_DOWN
                class="cn-native-select-icon"
                attr:data-slot="native-select-icon"
            />
        </div>
    }
}

#[component(element = "option")]
pub fn NativeSelectOption(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <option data-slot="native-select-option" {..attrs}>
            {children.map(|children| children())}
        </option>
    }
}

#[component(element = "optgroup")]
pub fn NativeSelectOptGroup(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <optgroup data-slot="native-select-optgroup" {..attrs}>
            {children.map(|children| children())}
        </optgroup>
    }
}

#[cfg(debug_assertions)]
fn check(size: &str) {
    const SIZES: &[&str] = &["default", "sm"];
    if !SIZES.contains(&size) {
        leptos::logging::warn!("NativeSelect: unknown size `{size}`, expected one of {SIZES:?}");
    }
}
