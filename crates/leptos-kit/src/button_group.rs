use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "div", skip(role))]
pub fn ButtonGroup(
    #[prop(optional, into)] orientation: Option<Signal<&'static str>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        let orientation = orientation.map_or("horizontal", |orientation| orientation.get());
        #[cfg(debug_assertions)]
        check("ButtonGroup", orientation);
        format!("cn-button-group cn-button-group-orientation-{orientation}")
    };
    view! {
        <div
            class=classes(own, class)
            role="group"
            data-orientation=move || orientation.map(|orientation| orientation.get())
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
    #[prop(into, default = "vertical".into())] orientation: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
) -> impl IntoView {
    let orientation = move || {
        #[cfg(debug_assertions)]
        check("ButtonGroupSeparator", orientation.get());
        orientation.get()
    };
    view! {
        <div
            class=classes("cn-button-group-separator", class)
            role="separator"
            aria-orientation=orientation
            data-orientation=orientation
            data-slot="button-group-separator"
            {..attrs}
        />
    }
}

#[cfg(debug_assertions)]
fn check(part: &str, orientation: &str) {
    const ORIENTATIONS: &[&str] = &["horizontal", "vertical"];
    if !ORIENTATIONS.contains(&orientation) {
        leptos::logging::warn!(
            "{part}: unknown orientation `{orientation}`, expected one of {ORIENTATIONS:?}"
        );
    }
}
