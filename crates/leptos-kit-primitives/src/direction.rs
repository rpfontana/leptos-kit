use crate::Direction;
use leptos::context::Provider;
use leptos::prelude::*;

#[derive(Clone, Copy)]
struct DirectionContext(Signal<Direction>);

#[leptos::component]
pub fn DirectionProvider(
    #[prop(optional, into)] direction: Signal<Direction>,
    children: Children,
) -> impl IntoView {
    view! { <Provider value=DirectionContext(direction)>{children()}</Provider> }
}

pub fn use_direction() -> Signal<Direction> {
    use_context::<DirectionContext>().map_or_else(|| Direction::Ltr.into(), |direction| direction.0)
}
