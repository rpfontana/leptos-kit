use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

primitives::keywords! {
    #[derive(Default)]
    pub enum MarkerVariant {
        #[default]
        Default = "default",
        Separator = "separator",
        Border = "border",
    }
}

#[component(element = "div")]
pub fn Marker(
    #[prop(into, default = MarkerVariant::Default.into())] variant: Signal<MarkerVariant>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        format!(
            "cn-marker group/marker cn-marker-variant-{}",
            variant.get().as_str()
        )
    };
    view! {
        <div class=classes(own, class) data-variant=primitives::keyword(variant) data-slot="marker" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "span")]
pub fn MarkerIcon(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <span
            class=classes("cn-marker-icon", class)
            data-slot="marker-icon"
            aria-hidden="true"
            {..attrs}
        >
            {children.map(|children| children())}
        </span>
    }
}

#[component(element = "span")]
pub fn MarkerContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <span class=classes("cn-marker-content", class) data-slot="marker-content" {..attrs}>
            {children.map(|children| children())}
        </span>
    }
}
