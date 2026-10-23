use super::button::button_look;
use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

pub use primitives::PreviewCardRoot as HoverCard;

#[component(element = "a")]
pub fn HoverCardTrigger(
    #[prop(optional, into)] variant: Option<Signal<&'static str>>,
    #[prop(optional, into)] size: Option<Signal<&'static str>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] delay: Option<Signal<u32>>,
    #[prop(optional, into)] close_delay: Option<Signal<u32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    // a button's look makes it a button, as shadcn's `render={<Button …/>}`
    let native_button = variant.is_some() || size.is_some();
    view! {
        <primitives::PreviewCardTrigger
            delay=delay
            close_delay=close_delay
            native_button=native_button
            attr:data-slot="hover-card-trigger"
            {..button_look(variant, size, class)}
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::PreviewCardTrigger>
    }
}

#[component(element = "div")]
pub fn HoverCardContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into, default = "bottom".into())] side: Signal<&'static str>,
    #[prop(into, default = 4.0.into())] side_offset: Signal<f64>,
    #[prop(into, default = "center".into())] align: Signal<&'static str>,
    #[prop(into, default = 4.0.into())] align_offset: Signal<f64>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::PreviewCardPositioner side align side_offset align_offset class="isolate z-50">
            <primitives::PreviewCardPopup
                class=classes("cn-hover-card-content cn-hover-card-content-logical", class)
                attr:data-slot="hover-card-content"
                {..attrs}
            >
                {children.map(|children| children())}
            </primitives::PreviewCardPopup>
        </primitives::PreviewCardPositioner>
    }
}
