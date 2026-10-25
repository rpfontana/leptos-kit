use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[component(element = "div", skip(role))]
pub fn Separator(
    #[prop(into, default = primitives::Orientation::Horizontal.into())] orientation: Signal<
        primitives::Orientation,
    >,
) -> impl IntoView {
    view! {
        <div
            role="separator"
            aria-orientation=primitives::keyword(orientation)
            data-orientation=primitives::keyword(orientation)
            data-slot="separator"
            {..attrs}
        />
    }
}
