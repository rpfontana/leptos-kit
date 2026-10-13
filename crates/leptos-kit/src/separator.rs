use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "div", skip(role))]
pub fn Separator(
    #[prop(into, default = "horizontal".into())] orientation: Signal<&'static str>,
) -> impl IntoView {
    let orientation = move || {
        #[cfg(debug_assertions)]
        check(orientation.get());
        orientation.get()
    };
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
