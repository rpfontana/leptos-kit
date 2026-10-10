use leptos::either::Either;
use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};

#[component]
pub fn Spinner(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] label: Option<Signal<String>>,
) -> impl IntoView {
    let class = match class {
        Some(class) => Either::Left(leptos::tachys::html::class::class(class)),
        None => Either::Right(()),
    };
    view! {
        <Icon
            icon=semantic::SPINNER_LOADER_2
            label=label.unwrap_or_else(|| "Loading".into())
            role="status"
            attr:data-slot="spinner"
            {..class}
        />
    }
}
