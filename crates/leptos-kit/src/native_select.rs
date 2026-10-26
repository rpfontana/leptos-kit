use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

primitives::keywords! {
    #[derive(Default)]
    #[non_exhaustive]
    pub enum NativeSelectSize {
        #[default]
        Default = "default",
        Sm = "sm",
    }
}

#[component(element = "select")]
pub fn NativeSelect(
    #[prop(into, default = NativeSelectSize::Default.into())] size: Signal<NativeSelectSize>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div
            class=classes("cn-native-select-wrapper group/native-select", class)
            data-slot="native-select-wrapper"
            data-size=primitives::keyword(size)
        >
            <select class="cn-native-select" data-size=primitives::keyword(size) data-slot="native-select" {..attrs}>
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
