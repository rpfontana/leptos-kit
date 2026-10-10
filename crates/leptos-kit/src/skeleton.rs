use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "div")]
pub fn Skeleton(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-skeleton", class) data-slot="skeleton" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}
