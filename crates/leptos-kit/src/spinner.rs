use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};

#[component]
pub fn Spinner(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] label: Option<Signal<String>>,
) -> impl IntoView {
    view! {
        <Icon
            icon=semantic::SPINNER_LOADER_2
            class=Signal::derive(move || class.map(|class| class.get()))
            label=Signal::derive(move || label.map_or_else(|| "Loading".to_string(), |label| label.get()))
            role="status"
            attr:data-slot="spinner"
        />
    }
}
