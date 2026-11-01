use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};
use leptos_kit_macros::component;
use leptos_kit_primitives::{self as primitives, Align, ChangeReason, Orientation, Side};

pub fn navigation_menu_trigger_style() -> &'static str {
    "cn-navigation-menu-trigger group/navigation-menu-trigger"
}

#[component(element = "nav")]
pub fn NavigationMenu(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] value: Option<Signal<Option<String>>>,
    #[prop(optional, into)] default_value: Option<String>,
    #[prop(optional, into)] on_value_change: Option<Callback<(Option<String>, ChangeReason)>>,
    #[prop(into, default = 50.into())] delay: Signal<u32>,
    #[prop(into, default = 50.into())] close_delay: Signal<u32>,
    #[prop(into, default = Orientation::Horizontal.into())] orientation: Signal<Orientation>,
    #[prop(optional, into)] on_open_change_complete: Option<Callback<bool>>,
    #[prop(into, default = Align::Start.into())] align: Signal<Align>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::NavigationMenuRoot
            nostrip:value=value
            nostrip:default_value=default_value
            nostrip:on_value_change=on_value_change
            delay=delay
            close_delay=close_delay
            orientation=orientation
            nostrip:on_open_change_complete=on_open_change_complete
            class=classes("cn-navigation-menu group/navigation-menu", class)
            attr:data-slot="navigation-menu"
            {..attrs}
        >
            {children.map(|children| children())}
            <NavigationMenuPositioner align=align />
        </primitives::NavigationMenuRoot>
    }
}

#[component(element = "ul")]
pub fn NavigationMenuList(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::NavigationMenuList
            class=classes("cn-navigation-menu-list group", class)
            attr:data-slot="navigation-menu-list"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::NavigationMenuList>
    }
}

#[component(element = "li")]
pub fn NavigationMenuItem(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] value: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::NavigationMenuItem
            nostrip:value=value
            class=classes("cn-navigation-menu-item", class)
            attr:data-slot="navigation-menu-item"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::NavigationMenuItem>
    }
}

#[component(element = "button", skip(r#type))]
pub fn NavigationMenuTrigger(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::NavigationMenuTrigger
            disabled=disabled
            class=classes(|| format!("{} group", navigation_menu_trigger_style()), class)
            attr:data-slot="navigation-menu-trigger"
            {..attrs}
        >
            // shadcn's {" "} collapses in the flex trigger, and a whitespace-only text node breaks islands hydration
            {children.map(|children| children())}
            <Icon
                icon=semantic::NAVIGATION_MENU_CHEVRON_DOWN
                class="cn-navigation-menu-trigger-icon"
            />
        </primitives::NavigationMenuTrigger>
    }
}

#[component(element = "div")]
pub fn NavigationMenuContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::NavigationMenuContent
            class=classes("cn-navigation-menu-content", class)
            attr:data-slot="navigation-menu-content"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::NavigationMenuContent>
    }
}

#[component(element = "div", skip(role, style))]
pub fn NavigationMenuPositioner(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into, default = Side::Bottom.into())] side: Signal<Side>,
    #[prop(into, default = 8.0.into())] side_offset: Signal<f64>,
    #[prop(into, default = Align::Start.into())] align: Signal<Align>,
    #[prop(optional, into)] align_offset: Signal<f64>,
) -> impl IntoView {
    view! {
        <primitives::NavigationMenuPortal>
            <primitives::NavigationMenuPositioner
                side=side
                side_offset=side_offset
                align=align
                align_offset=align_offset
                class=classes("cn-navigation-menu-positioner", class)
                {..attrs}
            >
                <primitives::NavigationMenuPopup class="cn-navigation-menu-popup">
                    <primitives::NavigationMenuViewport class="relative size-full overflow-hidden" />
                </primitives::NavigationMenuPopup>
            </primitives::NavigationMenuPositioner>
        </primitives::NavigationMenuPortal>
    }
}

#[component(element = "a")]
pub fn NavigationMenuLink(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] active: Signal<bool>,
    #[prop(optional)] close_on_click: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::NavigationMenuLink
            active=active
            close_on_click=close_on_click
            class=classes("cn-navigation-menu-link", class)
            attr:data-slot="navigation-menu-link"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::NavigationMenuLink>
    }
}

#[component(element = "span")]
pub fn NavigationMenuIndicator(
    #[prop(optional, into)] class: Option<Signal<String>>,
) -> impl IntoView {
    view! {
        <primitives::NavigationMenuIcon
            class=classes("cn-navigation-menu-indicator", class)
            attr:data-slot="navigation-menu-indicator"
            {..attrs}
        >
            <div class="cn-navigation-menu-indicator-arrow"></div>
        </primitives::NavigationMenuIcon>
    }
}
