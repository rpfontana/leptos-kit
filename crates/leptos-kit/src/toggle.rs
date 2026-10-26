use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

primitives::keywords! {
    #[derive(Default)]
    #[non_exhaustive]
    pub enum ToggleVariant {
        #[default]
        Default = "default",
        Outline = "outline",
    }
}

primitives::keywords! {
    #[derive(Default)]
    #[non_exhaustive]
    pub enum ToggleSize {
        #[default]
        Default = "default",
        Sm = "sm",
        Lg = "lg",
    }
}

#[component(element = "button", skip(r#type, form, value))]
pub fn Toggle(
    #[prop(into, default = ToggleVariant::Default.into())] variant: Signal<ToggleVariant>,
    #[prop(into, default = ToggleSize::Default.into())] size: Signal<ToggleSize>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] pressed: Option<Signal<bool>>,
    #[prop(optional)] default_pressed: bool,
    #[prop(optional, into)] on_pressed_change: Option<Callback<(bool, primitives::ChangeReason)>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional, into)] value: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || toggle_classes(variant.get(), size.get());
    view! {
        <button
            {..attrs}
            {..primitives::toggle_attributes(pressed, default_pressed, on_pressed_change, disabled, tabindex, value)}
            class=classes(own, class)
            data-slot="toggle"
        >
            {children.map(|children| children())}
        </button>
    }
}

pub(crate) fn toggle_classes(variant: ToggleVariant, size: ToggleSize) -> String {
    format!(
        "cn-toggle group/toggle cn-toggle-variant-{} cn-toggle-size-{}",
        variant.as_str(),
        size.as_str()
    )
}
