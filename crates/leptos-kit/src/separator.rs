use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "div", skip(role))]
pub fn Separator(#[prop(default = "horizontal")] orientation: &'static str) -> impl IntoView {
    #[cfg(debug_assertions)]
    check(orientation);
    view! {
        <div
            role="separator"
            aria-orientation=orientation
            data-orientation=orientation
            data-slot="separator"
            {..attrs}
        />
    }
}

#[cfg(debug_assertions)]
fn check(orientation: &str) {
    const ORIENTATIONS: &[&str] = &["horizontal", "vertical"];
    if !ORIENTATIONS.contains(&orientation) {
        leptos::logging::warn!(
            "Separator: unknown orientation `{orientation}`, expected one of {ORIENTATIONS:?}"
        );
    }
}
