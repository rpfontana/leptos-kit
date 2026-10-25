use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[component(element = "div", skip(role))]
pub fn ButtonGroup(
    #[prop(optional, into)] orientation: Option<Signal<primitives::Orientation>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        let orientation = orientation
            .map_or(primitives::Orientation::Horizontal, |orientation| {
                orientation.get()
            })
            .as_str();
        format!("cn-button-group cn-button-group-orientation-{orientation}")
    };
    view! {
        <div
            class=classes(own, class)
            role="group"
            data-orientation=move || orientation.map(|orientation| orientation.get().as_str())
            data-slot="button-group"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn ButtonGroupText(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-button-group-text", class) data-slot="button-group-text" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div", skip(role))]
pub fn ButtonGroupSeparator(
    #[prop(into, default = primitives::Orientation::Vertical.into())] orientation: Signal<
        primitives::Orientation,
    >,
    #[prop(optional, into)] class: Option<Signal<String>>,
) -> impl IntoView {
    view! {
        <div
            class=classes("cn-button-group-separator", class)
            role="separator"
            aria-orientation=primitives::keyword(orientation)
            data-orientation=primitives::keyword(orientation)
            data-slot="button-group-separator"
            {..attrs}
        />
    }
}
