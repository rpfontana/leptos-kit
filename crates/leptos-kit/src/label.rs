use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "label")]
pub fn Label(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <label class=classes("cn-label", class) data-slot="label" {..attrs}>
            {children.map(|children| children())}
        </label>
    }
}
