use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[component(element = "div", skip(role))]
pub fn ScrollArea(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] overflow_edge_threshold: Signal<f64>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ScrollAreaRoot
            overflow_edge_threshold=overflow_edge_threshold
            class=classes("cn-scroll-area", class)
            attr:data-slot="scroll-area"
            {..attrs}
        >
            <primitives::ScrollAreaViewport
                class="cn-scroll-area-viewport"
                attr:data-slot="scroll-area-viewport"
            >
                {children.map(|children| children())}
            </primitives::ScrollAreaViewport>
            <ScrollBar />
            <primitives::ScrollAreaCorner />
        </primitives::ScrollAreaRoot>
    }
}

#[component(element = "div")]
pub fn ScrollBar(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into, default = "vertical".into())] orientation: Signal<&'static str>,
    #[prop(optional, into)] keep_mounted: Signal<bool>,
) -> impl IntoView {
    view! {
        <primitives::ScrollAreaScrollbar
            orientation=orientation
            keep_mounted=keep_mounted
            class=classes("cn-scroll-area-scrollbar", class)
            attr:data-slot="scroll-area-scrollbar"
            {..attrs}
        >
            <primitives::ScrollAreaThumb
                class="cn-scroll-area-thumb"
                attr:data-slot="scroll-area-thumb"
            />
        </primitives::ScrollAreaScrollbar>
    }
}
