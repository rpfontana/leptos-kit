use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "textarea")]
pub fn Textarea(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <textarea class=classes(TEXTAREA_CLASS, class) data-slot="textarea" {..attrs}>
            {children.map(|children| children())}
        </textarea>
    }
}

pub(crate) const TEXTAREA_CLASS: &str = "cn-textarea";
