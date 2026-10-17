use super::toggle::toggle_classes;
use super::utils::classes;
use leptos::context::Provider;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[derive(Clone, Copy)]
struct Look {
    variant: Option<Signal<&'static str>>,
    size: Option<Signal<&'static str>>,
    spacing: Signal<f64>,
}

#[component(element = "div", skip(role))]
pub fn ToggleGroup(
    #[prop(optional, into)] variant: Option<Signal<&'static str>>,
    #[prop(optional, into)] size: Option<Signal<&'static str>>,
    #[prop(into, default = 2.0.into())] spacing: Signal<f64>,
    #[prop(into, default = "horizontal".into())] orientation: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] value: Option<Signal<Vec<String>>>,
    #[prop(optional)] default_value: Vec<String>,
    #[prop(optional, into)] on_value_change: Option<Callback<(Vec<String>, &'static str)>>,
    #[prop(optional, into)] multiple: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(into, default = true.into())] loop_focus: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let look = Look {
        variant,
        size,
        spacing,
    };
    view! {
        <primitives::ToggleGroup
            value=value
            default_value=default_value
            on_value_change=on_value_change
            multiple=multiple
            disabled=disabled
            orientation=orientation
            loop_focus=loop_focus
            class=classes("cn-toggle-group group/toggle-group", class)
            attr:data-slot="toggle-group"
            attr:data-variant=move || variant.map(|variant| variant.get())
            attr:data-size=move || size.map(|size| size.get())
            attr:data-spacing=move || spacing.get().to_string()
            style:--gap=move || spacing.get().to_string()
            {..attrs}
        >
            <Provider value=look>{children.map(|children| children())}</Provider>
        </primitives::ToggleGroup>
    }
}

#[component(element = "button", skip(r#type, form, value))]
pub fn ToggleGroupItem(
    #[prop(into, default = "default".into())] variant: Signal<&'static str>,
    #[prop(into, default = "default".into())] size: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] value: Option<String>,
    #[prop(optional, into)] on_pressed_change: Option<Callback<(bool, &'static str)>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let look = use_context::<Look>();
    let variant = move || look.and_then(|look| look.variant).unwrap_or(variant).get();
    let size = move || look.and_then(|look| look.size).unwrap_or(size).get();
    let own = move || format!("cn-toggle-group-item {}", toggle_classes(variant(), size()));
    view! {
        <button
            {..attrs}
            {..primitives::toggle_attributes(None, false, on_pressed_change, disabled, None, value)}
            class=classes(own, class)
            data-slot="toggle-group-item"
            data-variant=variant
            data-size=size
            data-spacing=move || look.map(|look| look.spacing.get().to_string())
        >
            {children.map(|children| children())}
        </button>
    }
}
