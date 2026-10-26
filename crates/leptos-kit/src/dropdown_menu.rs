use super::button::{ButtonSize, ButtonVariant, button_look};
use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
#[non_exhaustive]
pub enum DropdownMenuItemVariant {
    #[default]
    Default,
    Destructive,
}

pub use primitives::MenuPortal as DropdownMenuPortal;
pub use primitives::MenuRoot as DropdownMenu;
pub use primitives::MenuSubmenuRoot as DropdownMenuSub;

#[component(
    element = "button",
    skip(r#type, id, popovertarget, popovertargetaction)
)]
pub fn DropdownMenuTrigger(
    #[prop(optional, into)] variant: Option<Signal<ButtonVariant>>,
    #[prop(optional, into)] size: Option<Signal<ButtonSize>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] open_on_hover: Signal<bool>,
    #[prop(into, default = 100.into())] delay: Signal<u32>,
    #[prop(optional, into)] close_delay: Signal<u32>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::MenuTrigger
            disabled=disabled
            id=id
            open_on_hover=open_on_hover
            delay=delay
            close_delay=close_delay
            attr:data-slot="dropdown-menu-trigger"
            {..button_look(variant, size, class)}
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::MenuTrigger>
    }
}

#[component(element = "div")]
pub fn DropdownMenuContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into, default = primitives::Align::Start.into())] align: Signal<primitives::Align>,
    #[prop(optional, into)] align_offset: Signal<f64>,
    #[prop(into, default = primitives::Side::Bottom.into())] side: Signal<primitives::Side>,
    #[prop(into, default = 4.0.into())] side_offset: Signal<f64>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::MenuPositioner side align side_offset align_offset class="isolate z-50 outline-none">
            <primitives::MenuPopup
                class=classes("cn-dropdown-menu-content cn-dropdown-menu-content-logical", class)
                attr:data-slot="dropdown-menu-content"
                {..attrs}
            >
                {children.map(|children| children())}
            </primitives::MenuPopup>
        </primitives::MenuPositioner>
    }
}

#[component(element = "div")]
pub fn DropdownMenuSubContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into, default = primitives::Align::Start.into())] align: Signal<primitives::Align>,
    #[prop(into, default = (-3.0).into())] align_offset: Signal<f64>,
    #[prop(into, default = primitives::Side::Right.into())] side: Signal<primitives::Side>,
    #[prop(optional, into)] side_offset: Signal<f64>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::MenuPositioner side align side_offset align_offset class="isolate z-50 outline-none">
            <primitives::MenuPopup
                class=classes(
                    "cn-dropdown-menu-content cn-dropdown-menu-content-logical cn-dropdown-menu-sub-content",
                    class,
                )
                attr:data-slot="dropdown-menu-sub-content"
                {..attrs}
            >
                {children.map(|children| children())}
            </primitives::MenuPopup>
        </primitives::MenuPositioner>
    }
}

#[component(element = "div", skip(role))]
pub fn DropdownMenuGroup(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <primitives::MenuGroup attr:data-slot="dropdown-menu-group" {..attrs}>
            {children.map(|children| children())}
        </primitives::MenuGroup>
    }
}

#[component(element = "div", skip(id))]
pub fn DropdownMenuLabel(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] inset: Signal<bool>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::MenuGroupLabel
            id=id
            class=classes("cn-dropdown-menu-label", class)
            attr:data-slot="dropdown-menu-label"
            attr:data-inset=move || inset.get().then_some("true")
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::MenuGroupLabel>
    }
}

#[component(element = "div", skip(role, tabindex))]
pub fn DropdownMenuItem(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] inset: Signal<bool>,
    #[prop(into, default = DropdownMenuItemVariant::Default.into())] variant: Signal<
        DropdownMenuItemVariant,
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
            class=classes("cn-dropdown-menu-item group/dropdown-menu-item", class)
            attr:data-slot="dropdown-menu-item"
            attr:data-inset=move || inset.get().then_some("true")
            attr:data-variant=primitives::keyword(variant)
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::MenuItem>
    }
}

#[component(element = "div", skip(role, tabindex, id))]
pub fn DropdownMenuSubTrigger(
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
            class=classes("cn-dropdown-menu-sub-trigger", class)
            attr:data-slot="dropdown-menu-sub-trigger"
            attr:data-inset=move || inset.get().then_some("true")
            {..attrs}
        >
            {children.map(|children| children())}
            <Icon icon=semantic::DROPDOWN_MENU_CHEVRON_RIGHT class="cn-rtl-flip ml-auto" />
        </primitives::MenuSubmenuTrigger>
    }
}

#[component(element = "div", skip(role, tabindex))]
pub fn DropdownMenuCheckboxItem(
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
            class=classes("cn-dropdown-menu-checkbox-item", class)
            attr:data-slot="dropdown-menu-checkbox-item"
            attr:data-inset=move || inset.get().then_some("true")
            {..attrs}
        >
            <span
                class="cn-dropdown-menu-item-indicator"
                data-slot="dropdown-menu-checkbox-item-indicator"
            >
                <primitives::MenuCheckboxItemIndicator>
                    <Icon icon=semantic::DROPDOWN_MENU_CHECK />
                </primitives::MenuCheckboxItemIndicator>
            </span>
            {children.map(|children| children())}
        </primitives::MenuCheckboxItem>
    }
}

#[component(element = "div", skip(role))]
pub fn DropdownMenuRadioGroup(
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
            attr:data-slot="dropdown-menu-radio-group"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::MenuRadioGroup>
    }
}

#[component(element = "div", skip(role, tabindex))]
pub fn DropdownMenuRadioItem(
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
            class=classes("cn-dropdown-menu-radio-item", class)
            attr:data-slot="dropdown-menu-radio-item"
            attr:data-inset=move || inset.get().then_some("true")
            {..attrs}
        >
            <span
                class="cn-dropdown-menu-item-indicator"
                data-slot="dropdown-menu-radio-item-indicator"
            >
                <primitives::MenuRadioItemIndicator>
                    <Icon icon=semantic::DROPDOWN_MENU_CHECK />
                </primitives::MenuRadioItemIndicator>
            </span>
            {children.map(|children| children())}
        </primitives::MenuRadioItem>
    }
}

#[component(element = "div", skip(role))]
pub fn DropdownMenuSeparator(
    #[prop(optional, into)] class: Option<Signal<String>>,
) -> impl IntoView {
    view! {
        <div
            {..attrs}
            class=classes("cn-dropdown-menu-separator", class)
            role="separator"
            aria-orientation="horizontal"
            data-orientation="horizontal"
            data-slot="dropdown-menu-separator"
        ></div>
    }
}

#[component(element = "span")]
pub fn DropdownMenuShortcut(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <span {..attrs} class=classes("cn-dropdown-menu-shortcut", class) data-slot="dropdown-menu-shortcut">
            {children.map(|children| children())}
        </span>
    }
}
