use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

primitives::keywords! {
    #[derive(Default)]
    #[non_exhaustive]
    pub enum ContextMenuItemVariant {
        #[default]
        Default = "default",
        Destructive = "destructive",
    }
}

pub use primitives::ContextMenuRoot as ContextMenu;
pub use primitives::MenuPortal as ContextMenuPortal;
pub use primitives::MenuSubmenuRoot as ContextMenuSub;

#[component(element = "div")]
pub fn ContextMenuTrigger(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ContextMenuTrigger
            class=classes("cn-context-menu-trigger", class)
            attr:data-slot="context-menu-trigger"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::ContextMenuTrigger>
    }
}

#[component(element = "div")]
pub fn ContextMenuContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into, default = primitives::Align::Start.into())] align: Signal<primitives::Align>,
    #[prop(into, default = 4.0.into())] align_offset: Signal<f64>,
    #[prop(into, default = primitives::Side::Right.into())] side: Signal<primitives::Side>,
    #[prop(optional, into)] side_offset: Signal<f64>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::MenuPositioner side align side_offset align_offset class="isolate z-50 outline-none">
            <primitives::MenuPopup
                class=classes("cn-context-menu-content cn-context-menu-content-logical", class)
                attr:data-slot="context-menu-content"
                {..attrs}
            >
                {children.map(|children| children())}
            </primitives::MenuPopup>
        </primitives::MenuPositioner>
    }
}

#[component(element = "div")]
pub fn ContextMenuSubContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into, default = primitives::Align::Start.into())] align: Signal<primitives::Align>,
    #[prop(into, default = 4.0.into())] align_offset: Signal<f64>,
    #[prop(into, default = primitives::Side::Right.into())] side: Signal<primitives::Side>,
    #[prop(optional, into)] side_offset: Signal<f64>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::MenuPositioner side align side_offset align_offset class="isolate z-50 outline-none">
            <primitives::MenuPopup
                class=classes(
                    "cn-context-menu-content cn-context-menu-content-logical cn-context-menu-subcontent",
                    class,
                )
                attr:data-slot="context-menu-sub-content"
                {..attrs}
            >
                {children.map(|children| children())}
            </primitives::MenuPopup>
        </primitives::MenuPositioner>
    }
}

#[component(element = "div", skip(role))]
pub fn ContextMenuGroup(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <primitives::MenuGroup attr:data-slot="context-menu-group" {..attrs}>
            {children.map(|children| children())}
        </primitives::MenuGroup>
    }
}

#[component(element = "div", skip(id))]
pub fn ContextMenuLabel(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] inset: Signal<bool>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::MenuGroupLabel
            id=id
            class=classes("cn-context-menu-label", class)
            attr:data-slot="context-menu-label"
            attr:data-inset=move || inset.get().then_some("true")
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::MenuGroupLabel>
    }
}

#[component(element = "div", skip(role, tabindex))]
pub fn ContextMenuItem(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] inset: Signal<bool>,
    #[prop(into, default = ContextMenuItemVariant::Default.into())] variant: Signal<
        ContextMenuItemVariant,
    >,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(into, default = true.into())] close_on_click: Signal<bool>,
    #[prop(optional, into)] label: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::MenuItem
            disabled=disabled
            close_on_click=close_on_click
            label=label
            class=classes("cn-context-menu-item group/context-menu-item", class)
            attr:data-slot="context-menu-item"
            attr:data-inset=move || inset.get().then_some("true")
            attr:data-variant=primitives::keyword(variant)
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::MenuItem>
    }
}

#[component(element = "div", skip(role, tabindex, id))]
pub fn ContextMenuSubTrigger(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] inset: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(into, default = true.into())] open_on_hover: Signal<bool>,
    #[prop(into, default = 100.into())] delay: Signal<u32>,
    #[prop(optional, into)] close_delay: Signal<u32>,
    #[prop(optional, into)] label: Option<Signal<String>>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::MenuSubmenuTrigger
            id=id
            disabled=disabled
            open_on_hover=open_on_hover
            delay=delay
            close_delay=close_delay
            label=label
            class=classes("cn-context-menu-sub-trigger", class)
            attr:data-slot="context-menu-sub-trigger"
            attr:data-inset=move || inset.get().then_some("true")
            {..attrs}
        >
            {children.map(|children| children())}
            <Icon icon=semantic::CONTEXT_MENU_CHEVRON_RIGHT class="cn-rtl-flip ml-auto" />
        </primitives::MenuSubmenuTrigger>
    }
}

#[component(element = "div", skip(role, tabindex))]
pub fn ContextMenuCheckboxItem(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] checked: Option<Signal<bool>>,
    #[prop(optional)] default_checked: bool,
    #[prop(optional, into)] on_checked_change: Option<Callback<(bool, primitives::ChangeReason)>>,
    #[prop(optional, into)] inset: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] close_on_click: Signal<bool>,
    #[prop(optional, into)] label: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::MenuCheckboxItem
            checked=checked
            default_checked=default_checked
            on_checked_change=on_checked_change
            disabled=disabled
            close_on_click=close_on_click
            label=label
            class=classes("cn-context-menu-checkbox-item", class)
            attr:data-slot="context-menu-checkbox-item"
            attr:data-inset=move || inset.get().then_some("true")
            {..attrs}
        >
            <span class="cn-context-menu-item-indicator">
                <primitives::MenuCheckboxItemIndicator>
                    <Icon icon=semantic::CONTEXT_MENU_CHECK />
                </primitives::MenuCheckboxItemIndicator>
            </span>
            {children.map(|children| children())}
        </primitives::MenuCheckboxItem>
    }
}

#[component(element = "div", skip(role))]
pub fn ContextMenuRadioGroup(
    #[prop(optional, into)] value: Option<Signal<Option<String>>>,
    #[prop(optional, into)] default_value: Option<String>,
    #[prop(optional, into)] on_value_change: Option<Callback<(String, primitives::ChangeReason)>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::MenuRadioGroup
            value=value
            default_value=default_value
            on_value_change=on_value_change
            disabled=disabled
            attr:data-slot="context-menu-radio-group"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::MenuRadioGroup>
    }
}

#[component(element = "div", skip(role, tabindex))]
pub fn ContextMenuRadioItem(
    #[prop(into)] value: String,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] inset: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] close_on_click: Signal<bool>,
    #[prop(optional, into)] label: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::MenuRadioItem
            value=value
            disabled=disabled
            close_on_click=close_on_click
            label=label
            class=classes("cn-context-menu-radio-item", class)
            attr:data-slot="context-menu-radio-item"
            attr:data-inset=move || inset.get().then_some("true")
            {..attrs}
        >
            <span class="cn-context-menu-item-indicator">
                <primitives::MenuRadioItemIndicator>
                    <Icon icon=semantic::CONTEXT_MENU_CHECK />
                </primitives::MenuRadioItemIndicator>
            </span>
            {children.map(|children| children())}
        </primitives::MenuRadioItem>
    }
}

#[component(element = "div", skip(role))]
pub fn ContextMenuSeparator(
    #[prop(optional, into)] class: Option<Signal<String>>,
) -> impl IntoView {
    view! {
        <div
            {..attrs}
            class=classes("cn-context-menu-separator", class)
            role="separator"
            aria-orientation="horizontal"
            data-orientation="horizontal"
            data-slot="context-menu-separator"
        ></div>
    }
}

#[component(element = "span")]
pub fn ContextMenuShortcut(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <span {..attrs} class=classes("cn-context-menu-shortcut", class) data-slot="context-menu-shortcut">
            {children.map(|children| children())}
        </span>
    }
}
