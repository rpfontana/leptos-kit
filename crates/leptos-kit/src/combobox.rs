use super::button::{ButtonSize, ButtonVariant, button_classes};
use super::input::INPUT_CLASS;
use super::input_group::{InputGroup, InputGroupAddon, InputGroupAddonAlign};
use super::utils::classes;
use leptos::html;
use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

pub use primitives::ComboboxCollection;
pub use primitives::ComboboxRoot as Combobox;
pub use primitives::ComboboxValue;

// what `<ComboboxChips node_ref>` and `<ComboboxContent anchor>` share
pub fn use_combobox_anchor() -> NodeRef<html::Div> {
    NodeRef::new()
}

// somehow like `render={<Button …/>}`
fn trigger_classes(
    variant: Option<Signal<ButtonVariant>>,
    size: Option<Signal<ButtonSize>>,
) -> String {
    if variant.is_none() && size.is_none() {
        return "cn-combobox-trigger".to_string();
    }
    let variant = variant.map_or(ButtonVariant::Default, |variant| variant.get());
    let size = size.map_or(ButtonSize::Default, |size| size.get());
    format!("cn-combobox-trigger {}", button_classes(variant, size))
}

// an input group's icon button, as `render={<InputGroupButton size="icon-xs" />}`
fn group_button(own: &str) -> String {
    format!(
        "{own} {} cn-input-group-button cn-input-group-button-size-icon-xs",
        button_classes(ButtonVariant::Ghost, ButtonSize::Default)
    )
}

#[component(element = "button", skip(r#type, id, role))]
pub fn ComboboxTrigger(
    #[prop(optional, into)] variant: Option<Signal<ButtonVariant>>,
    #[prop(optional, into)] size: Option<Signal<ButtonSize>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ComboboxTrigger
            disabled=disabled
            id=id
            class=classes(move || trigger_classes(variant, size), class)
            attr:data-slot="combobox-trigger"
            {..attrs}
        >
            {children.map(|children| children())}
            <Icon icon=semantic::COMBOBOX_CHEVRON_DOWN class="cn-combobox-trigger-icon" />
        </primitives::ComboboxTrigger>
    }
}

#[component(element = "button", skip(r#type))]
pub fn ComboboxClear(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
) -> impl IntoView {
    view! {
        <primitives::ComboboxClear
            disabled=disabled
            class=classes(|| group_button("cn-combobox-clear"), class)
            attr:data-slot="combobox-clear"
            attr:data-size="icon-xs"
            {..attrs}
        >
            <Icon icon=semantic::COMBOBOX_X class="cn-combobox-clear-icon" />
        </primitives::ComboboxClear>
    }
}

#[component(element = "input", skip(r#type, role, id, autocomplete))]
pub fn ComboboxInput(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(into, default = true)] show_trigger: bool,
    #[prop(optional)] show_clear: bool,
    #[prop(optional, into)] aria_invalid: Option<Signal<bool>>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let trigger = show_trigger.then(|| {
        // the clear stays in the dom while hidden, so only a shown one hides the trigger
        view! {
            <primitives::ComboboxTrigger
                disabled=Some(disabled)
                class=group_button(
                    "cn-combobox-trigger group-has-[[data-slot=combobox-clear]:not([hidden])]/input-group:hidden data-pressed:bg-transparent",
                )
                attr:data-slot="input-group-button"
                attr:data-size="icon-xs"
            >
                <Icon icon=semantic::COMBOBOX_CHEVRON_DOWN class="cn-combobox-trigger-icon" />
            </primitives::ComboboxTrigger>
        }
    });
    let clear = show_clear.then(|| view! { <ComboboxClear disabled=disabled /> });
    view! {
        <InputGroup class=classes("cn-combobox-input", class)>
            <primitives::ComboboxInput
                disabled=Some(disabled)
                id=id
                class=format!("{INPUT_CLASS} cn-input-group-input")
                attr:data-slot="input-group-control"
                attr:aria-invalid=move || aria_invalid.and_then(|invalid| invalid.get().then_some("true"))
                {..attrs}
            />
            <InputGroupAddon align=InputGroupAddonAlign::InlineEnd>{trigger}{clear}</InputGroupAddon>
            {children.map(|children| children())}
        </InputGroup>
    }
}

#[component(element = "div")]
pub fn ComboboxContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into, default = primitives::Side::Bottom.into())] side: Signal<primitives::Side>,
    #[prop(into, default = 6.0.into())] side_offset: Signal<f64>,
    #[prop(into, default = primitives::Align::Start.into())] align: Signal<primitives::Align>,
    #[prop(optional, into)] align_offset: Signal<f64>,
    #[prop(optional)] anchor: Option<NodeRef<html::Div>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ComboboxPortal>
            <primitives::ComboboxPositioner
                side
                side_offset
                align
                align_offset
                anchor=anchor
                class="isolate z-50"
            >
                <primitives::ComboboxPopup
                    class=classes(
                        "cn-combobox-content cn-combobox-content-logical group/combobox-content",
                        class,
                    )
                    attr:data-slot="combobox-content"
                    attr:data-chips=anchor.is_some().to_string()
                    {..attrs}
                >
                    {children.map(|children| children())}
                </primitives::ComboboxPopup>
            </primitives::ComboboxPositioner>
        </primitives::ComboboxPortal>
    }
}

#[component(element = "div", skip(id, role, tabindex))]
pub fn ComboboxList(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ComboboxList
            class=classes("cn-combobox-list", class)
            attr:data-slot="combobox-list"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::ComboboxList>
    }
}

#[component(element = "div", skip(id, role, tabindex))]
pub fn ComboboxItem(
    #[prop(into)] value: String,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ComboboxItem
            value=value
            disabled=disabled
            class=classes("cn-combobox-item", class)
            attr:data-slot="combobox-item"
            {..attrs}
        >
            {children.map(|children| children())}
            <primitives::ComboboxItemIndicator class="cn-combobox-item-indicator">
                <Icon icon=semantic::COMBOBOX_CHECK class="cn-combobox-item-indicator-icon" />
            </primitives::ComboboxItemIndicator>
        </primitives::ComboboxItem>
    }
}

#[component(element = "div", skip(role))]
pub fn ComboboxGroup(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] items: Option<Signal<Vec<String>>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ComboboxGroup
            items=items
            class=classes("cn-combobox-group", class)
            attr:data-slot="combobox-group"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::ComboboxGroup>
    }
}

#[component(element = "div", skip(id))]
pub fn ComboboxLabel(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ComboboxGroupLabel
            id=id
            class=classes("cn-combobox-label", class)
            attr:data-slot="combobox-label"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::ComboboxGroupLabel>
    }
}

#[component(element = "div", skip(role))]
pub fn ComboboxEmpty(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<ChildrenFn>,
) -> impl IntoView {
    view! {
        <primitives::ComboboxEmpty
            class=classes("cn-combobox-empty", class)
            attr:data-slot="combobox-empty"
            {..attrs}
        >
            {children.clone().map(|children| children())}
        </primitives::ComboboxEmpty>
    }
}

#[component(element = "div", skip(role))]
pub fn ComboboxSeparator(#[prop(optional, into)] class: Option<Signal<String>>) -> impl IntoView {
    view! {
        <primitives::ComboboxSeparator
            class=classes("cn-combobox-separator", class)
            attr:data-slot="combobox-separator"
            {..attrs}
        />
    }
}

#[component(element = "div", skip(role))]
pub fn ComboboxChips(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] node_ref: Option<NodeRef<html::Div>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ComboboxChips
            node_ref=node_ref
            class=classes("cn-combobox-chips", class)
            attr:data-slot="combobox-chips"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::ComboboxChips>
    }
}

#[component(element = "div", skip(tabindex))]
pub fn ComboboxChip(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into, default = true)] show_remove: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    // made inside the chip, which it reads
    let remove = move || {
        show_remove.then(|| {
            view! {
                <primitives::ComboboxChipRemove
                    class=format!(
                        "{} cn-combobox-chip-remove",
                        button_classes(ButtonVariant::Ghost, ButtonSize::IconXs),
                    )
                    attr:data-slot="combobox-chip-remove"
                >
                    <Icon icon=semantic::COMBOBOX_X class="cn-combobox-chip-indicator-icon" />
                </primitives::ComboboxChipRemove>
            }
        })
    };
    view! {
        <primitives::ComboboxChip
            class=classes("cn-combobox-chip", class)
            attr:data-slot="combobox-chip"
            {..attrs}
        >
            {children.map(|children| children())}
            {remove()}
        </primitives::ComboboxChip>
    }
}

#[component(element = "input", skip(r#type, role, id, autocomplete))]
pub fn ComboboxChipsInput(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] id: Option<String>,
) -> impl IntoView {
    view! {
        <primitives::ComboboxInput
            disabled=disabled
            id=id
            class=classes("cn-combobox-chip-input", class)
            attr:data-slot="combobox-chip-input"
            {..attrs}
        />
    }
}
