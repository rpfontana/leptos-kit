use super::button::{ButtonSize, ButtonVariant, button_look};
use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

pub use primitives::TooltipRoot as Tooltip;

#[leptos::component]
pub fn TooltipProvider(
    #[prop(into, default = 0.into())] delay: Signal<u32>,
    #[prop(optional, into)] close_delay: Option<Signal<u32>>,
    #[prop(into, default = 400.into())] timeout: Signal<u32>,
    children: Children,
) -> impl IntoView {
    view! {
        <primitives::TooltipProvider delay=delay close_delay=close_delay timeout=timeout>
            {children()}
        </primitives::TooltipProvider>
    }
}

#[component(element = "button", skip(r#type))]
pub fn TooltipTrigger(
    #[prop(optional, into)] variant: Option<Signal<ButtonVariant>>,
    #[prop(optional, into)] size: Option<Signal<ButtonSize>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] delay: Option<Signal<u32>>,
    #[prop(optional, into)] close_delay: Option<Signal<u32>>,
    #[prop(into, default = true.into())] close_on_click: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::TooltipTrigger
            delay=delay
            close_delay=close_delay
            close_on_click=close_on_click
            attr:data-slot="tooltip-trigger"
            {..button_look(variant, size, class)}
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::TooltipTrigger>
    }
}

#[component(element = "div")]
pub fn TooltipContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into, default = primitives::Side::Top.into())] side: Signal<primitives::Side>,
    #[prop(into, default = 4.0.into())] side_offset: Signal<f64>,
    #[prop(into, default = primitives::Align::Center.into())] align: Signal<primitives::Align>,
    #[prop(optional, into)] align_offset: Signal<f64>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::TooltipPositioner side align side_offset align_offset class="isolate z-50">
            <primitives::TooltipPopup
                class=classes("cn-tooltip-content cn-tooltip-content-logical", class)
                attr:data-slot="tooltip-content"
                {..attrs}
            >
                {children.map(|children| children())}
                <primitives::TooltipArrow class="cn-tooltip-arrow cn-tooltip-arrow-logical" />
            </primitives::TooltipPopup>
        </primitives::TooltipPositioner>
    }
}
