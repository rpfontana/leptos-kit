use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[component(element = "button", skip(r#type, form, value))]
pub fn Toggle(
    #[prop(into, default = "default".into())] variant: Signal<&'static str>,
    #[prop(into, default = "default".into())] size: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] pressed: Option<Signal<bool>>,
    #[prop(optional)] default_pressed: bool,
    #[prop(optional, into)] on_pressed_change: Option<Callback<(bool, &'static str)>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        #[cfg(debug_assertions)]
        check(variant.get(), size.get());
        format!(
            "cn-toggle group/toggle cn-toggle-variant-{} cn-toggle-size-{}",
            variant.get(),
            size.get()
        )
    };
    view! {
        <button
            {..attrs}
            {..primitives::toggle_attributes(pressed, default_pressed, on_pressed_change, disabled, tabindex)}
            class=classes(own, class)
            data-slot="toggle"
        >
            {children.map(|children| children())}
        </button>
    }
}

#[cfg(debug_assertions)]
fn check(variant: &str, size: &str) {
    const VARIANTS: &[&str] = &["default", "outline"];
    const SIZES: &[&str] = &["default", "sm", "lg"];
    if !VARIANTS.contains(&variant) {
        leptos::logging::warn!("Toggle: unknown variant `{variant}`, expected one of {VARIANTS:?}");
    }
    if !SIZES.contains(&size) {
        leptos::logging::warn!("Toggle: unknown size `{size}`, expected one of {SIZES:?}");
    }
}
