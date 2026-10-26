use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
#[non_exhaustive]
pub enum MenubarItemVariant {
    #[default]
    Default,
    Destructive,
}

pub use primitives::MenuPortal as MenubarPortal;
pub use primitives::MenuRoot as MenubarMenu;
pub use primitives::MenuSubmenuRoot as MenubarSub;

#[component(element = "div", skip(role, id))]
pub fn Menubar(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into, default = true.into())] modal: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(into, default = primitives::Orientation::Horizontal.into())] orientation: Signal<
        primitives::Orientation,
    >,
    #[prop(into, default = true.into())] loop_focus: Signal<bool>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::Menubar
            modal=modal
            disabled=disabled
            orientation=orientation
            loop_focus=loop_focus
            id=id
            class=classes("cn-menubar", class)
            attr:data-slot="menubar"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::Menubar>
    }
}

#[component(
    element = "button",
    skip(r#type, id, role, popovertarget, popovertargetaction)
)]
pub fn MenubarTrigger(
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
            class=classes("cn-menubar-trigger", class)
            attr:data-slot="menubar-trigger"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::MenuTrigger>
    }
}

#[component(element = "div")]
pub fn MenubarContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into, default = primitives::Align::Start.into())] align: Signal<primitives::Align>,
    #[prop(into, default = (-4.0).into())] align_offset: Signal<f64>,
    #[prop(into, default = primitives::Side::Bottom.into())] side: Signal<primitives::Side>,
    #[prop(into, default = 8.0.into())] side_offset: Signal<f64>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::MenuPositioner side align side_offset align_offset class="isolate z-50 outline-none">
            <primitives::MenuPopup
                class=classes(
                    "cn-dropdown-menu-content cn-dropdown-menu-content-logical cn-menubar-content cn-menubar-content-logical",
                    class,
                )
                attr:data-slot="menubar-content"
                {..attrs}
            >
                {children.map(|children| children())}
            </primitives::MenuPopup>
        </primitives::MenuPositioner>
    }
}

#[component(element = "div")]
pub fn MenubarSubContent(
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
                    "cn-dropdown-menu-content cn-dropdown-menu-content-logical cn-dropdown-menu-sub-content cn-menubar-sub-content",
                    class,
                )
                attr:data-slot="menubar-sub-content"
                {..attrs}
            >
                {children.map(|children| children())}
            </primitives::MenuPopup>
        </primitives::MenuPositioner>
    }
}

#[component(element = "div", skip(role))]
pub fn MenubarGroup(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <primitives::MenuGroup attr:data-slot="menubar-group" {..attrs}>
            {children.map(|children| children())}
        </primitives::MenuGroup>
    }
}

#[component(element = "div", skip(id))]
pub fn MenubarLabel(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] inset: Signal<bool>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::MenuGroupLabel
            id=id
            class=classes("cn-dropdown-menu-label cn-menubar-label", class)
            attr:data-slot="menubar-label"
            attr:data-inset=move || inset.get().then_some("true")
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::MenuGroupLabel>
    }
}

#[component(element = "div", skip(role, tabindex))]
pub fn MenubarItem(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] inset: Signal<bool>,
    #[prop(into, default = MenubarItemVariant::Default.into())] variant: Signal<MenubarItemVariant>,
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
            class=classes(
                "cn-dropdown-menu-item group/dropdown-menu-item cn-menubar-item group/menubar-item",
                class,
            )
            attr:data-slot="menubar-item"
            attr:data-inset=move || inset.get().then_some("true")
            attr:data-variant=primitives::keyword(variant)
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::MenuItem>
    }
}

#[component(element = "div", skip(role, tabindex, id))]
pub fn MenubarSubTrigger(
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
            class=classes("cn-dropdown-menu-sub-trigger cn-menubar-sub-trigger", class)
            attr:data-slot="menubar-sub-trigger"
            attr:data-inset=move || inset.get().then_some("true")
            {..attrs}
        >
            {children.map(|children| children())}
            <Icon icon=semantic::DROPDOWN_MENU_CHEVRON_RIGHT class="cn-rtl-flip ml-auto" />
        </primitives::MenuSubmenuTrigger>
    }
}

#[component(element = "div", skip(role, tabindex))]
pub fn MenubarCheckboxItem(
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
            class=classes("cn-menubar-checkbox-item", class)
            attr:data-slot="menubar-checkbox-item"
            attr:data-inset=move || inset.get().then_some("true")
            {..attrs}
        >
            <span class="cn-menubar-checkbox-item-indicator">
                <primitives::MenuCheckboxItemIndicator>
                    <Icon icon=semantic::MENUBAR_CHECK />
                </primitives::MenuCheckboxItemIndicator>
            </span>
            {children.map(|children| children())}
        </primitives::MenuCheckboxItem>
    }
}

#[component(element = "div", skip(role))]
pub fn MenubarRadioGroup(
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
            attr:data-slot="menubar-radio-group"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::MenuRadioGroup>
    }
}

#[component(element = "div", skip(role, tabindex))]
pub fn MenubarRadioItem(
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
            class=classes("cn-menubar-radio-item", class)
            attr:data-slot="menubar-radio-item"
            attr:data-inset=move || inset.get().then_some("true")
            {..attrs}
        >
            <span class="cn-menubar-radio-item-indicator">
                <primitives::MenuRadioItemIndicator>
                    <Icon icon=semantic::MENUBAR_CHECK />
                </primitives::MenuRadioItemIndicator>
            </span>
            {children.map(|children| children())}
        </primitives::MenuRadioItem>
    }
}

#[component(element = "div", skip(role))]
pub fn MenubarSeparator(#[prop(optional, into)] class: Option<Signal<String>>) -> impl IntoView {
    view! {
        <div
            {..attrs}
            class=classes("cn-dropdown-menu-separator cn-menubar-separator", class)
            role="separator"
            aria-orientation="horizontal"
            data-orientation="horizontal"
            data-slot="menubar-separator"
        ></div>
    }
}

#[component(element = "span")]
pub fn MenubarShortcut(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <span
            {..attrs}
            class=classes("cn-dropdown-menu-shortcut cn-menubar-shortcut", class)
            data-slot="menubar-shortcut"
        >
            {children.map(|children| children())}
        </span>
    }
}
