use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "kbd")]
pub fn Kbd(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <kbd class=classes("cn-kbd", class) data-slot="kbd" {..attrs}>
            {children.map(|children| children())}
        </kbd>
    }
}

#[component(element = "kbd")]
pub fn KbdGroup(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <kbd class=classes("cn-kbd-group", class) data-slot="kbd-group" {..attrs}>
            {children.map(|children| children())}
        </kbd>
    }
}
