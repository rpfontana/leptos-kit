use super::utils::classes;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
#[non_exhaustive]
pub enum SelectTriggerSize {
    Sm,
    #[default]
    Default,
}

pub use primitives::SelectRoot as Select;

#[component(element = "div", skip(role))]
pub fn SelectGroup(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::SelectGroup
            class=classes("cn-select-group", class)
            attr:data-slot="select-group"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::SelectGroup>
    }
}

#[component(element = "span")]
pub fn SelectValue(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] placeholder: MaybeProp<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    // without children it shows the selected label
    match children {
        Some(children) => Either::Left(view! {
            <primitives::SelectValue
                placeholder=placeholder
                class=classes("cn-select-value", class)
                attr:data-slot="select-value"
                {..attrs}
            >
                {children()}
            </primitives::SelectValue>
        }),
        None => Either::Right(view! {
            <primitives::SelectValue
                placeholder=placeholder
                class=classes("cn-select-value", class)
                attr:data-slot="select-value"
                {..attrs}
            />
        }),
    }
}

#[component(
    element = "button",
    skip(r#type, id, role, popovertarget, popovertargetaction)
)]
pub fn SelectTrigger(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into, default = SelectTriggerSize::Default.into())] size: Signal<SelectTriggerSize>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::SelectTrigger
            disabled=disabled
            id=id
            class=classes("cn-select-trigger", class)
            attr:data-slot="select-trigger"
            attr:data-size=primitives::keyword(size)
            {..attrs}
        >
            {children.map(|children| children())}
            <Icon
                icon=semantic::SELECT_CHEVRON_DOWN
                class="cn-select-trigger-icon"
                {..primitives::select_icon_attributes()}
            />
        </primitives::SelectTrigger>
    }
}

#[component(element = "div")]
pub fn SelectContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into, default = primitives::Side::Bottom.into())] side: Signal<primitives::Side>,
    #[prop(into, default = 4.0.into())] side_offset: Signal<f64>,
    #[prop(into, default = primitives::Align::Center.into())] align: Signal<primitives::Align>,
    #[prop(optional, into)] align_offset: Signal<f64>,
    #[prop(into, default = true.into())] align_item_with_trigger: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::SelectPortal>
            <primitives::SelectPositioner
                side
                side_offset
                align
                align_offset
                align_item_with_trigger
                class="isolate z-50"
            >
                <primitives::SelectPopup
                    class=classes("cn-select-content cn-select-content-logical", class)
                    attr:data-slot="select-content"
                    attr:data-align-trigger=move || align_item_with_trigger.get().to_string()
                    {..attrs}
                >
                    <SelectScrollUpButton />
                    <primitives::SelectList>{children.map(|children| children())}</primitives::SelectList>
                    <SelectScrollDownButton />
                </primitives::SelectPopup>
            </primitives::SelectPositioner>
        </primitives::SelectPortal>
    }
}

#[component(element = "div", skip(id))]
pub fn SelectLabel(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::SelectGroupLabel
            id=id
            class=classes("cn-select-label", class)
            attr:data-slot="select-label"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::SelectGroupLabel>
    }
}

#[component(element = "div", skip(role, tabindex))]
pub fn SelectItem(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] value: MaybeProp<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] label: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::SelectItem
            value=value
            disabled=disabled
            label=label
            class=classes("cn-select-item", class)
            attr:data-slot="select-item"
            {..attrs}
        >
            <primitives::SelectItemText class="cn-select-item-text">
                {children.map(|children| children())}
            </primitives::SelectItemText>
            <primitives::SelectItemIndicator class="cn-select-item-indicator">
                <Icon icon=semantic::SELECT_CHECK class="cn-select-item-indicator-icon" />
            </primitives::SelectItemIndicator>
        </primitives::SelectItem>
    }
}

#[component(element = "div", skip(role))]
pub fn SelectSeparator(#[prop(optional, into)] class: Option<Signal<String>>) -> impl IntoView {
    view! {
        <primitives::SelectSeparator
            class=classes("cn-select-separator", class)
            attr:data-slot="select-separator"
            {..attrs}
        />
    }
}

#[component(element = "div")]
pub fn SelectScrollUpButton(
    #[prop(optional, into)] class: Option<Signal<String>>,
) -> impl IntoView {
    view! {
        <primitives::SelectScrollUpArrow
            class=classes("cn-select-scroll-up-button", class)
            attr:data-slot="select-scroll-up-button"
            {..attrs}
        >
            <Icon icon=semantic::SELECT_CHEVRON_UP />
        </primitives::SelectScrollUpArrow>
    }
}

#[component(element = "div")]
pub fn SelectScrollDownButton(
    #[prop(optional, into)] class: Option<Signal<String>>,
) -> impl IntoView {
    view! {
        <primitives::SelectScrollDownArrow
            class=classes("cn-select-scroll-down-button", class)
            attr:data-slot="select-scroll-down-button"
            {..attrs}
        >
            <Icon icon=semantic::SELECT_CHEVRON_DOWN />
        </primitives::SelectScrollDownArrow>
    }
}
