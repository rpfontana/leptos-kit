use leptos::context::Provider;
use leptos::prelude::*;

#[derive(Clone, Copy)]
struct Direction(Signal<&'static str>);

#[leptos::component]
pub fn DirectionProvider(
    #[prop(into, default = "ltr".into())] direction: Signal<&'static str>,
    children: Children,
) -> impl IntoView {
    #[cfg(debug_assertions)]
    let direction = Signal::derive(move || {
        let direction = direction.get();
        if !["ltr", "rtl"].contains(&direction) {
            leptos::logging::warn!(
                "DirectionProvider: unknown direction `{direction}`, expected ltr or rtl"
            );
        }
        direction
    });
    view! { <Provider value=Direction(direction)>{children()}</Provider> }
}

pub fn use_direction() -> Signal<&'static str> {
    use_context::<Direction>().map_or_else(|| "ltr".into(), |direction| direction.0)
}
