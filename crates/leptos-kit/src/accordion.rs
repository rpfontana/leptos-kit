use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[component(element = "div")]
pub fn Accordion(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] value: Option<Signal<Vec<String>>>,
    #[prop(optional)] default_value: Vec<String>,
    #[prop(optional, into)] on_value_change: Option<Callback<(Vec<String>, &'static str)>>,
    #[prop(optional, into)] multiple: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(into, default = "vertical".into())] orientation: Signal<&'static str>,
    #[prop(optional, into)] hidden_until_found: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::AccordionRoot
            value=value
            default_value=default_value
            on_value_change=on_value_change
            multiple=multiple
            disabled=disabled
            orientation=orientation
            hidden_until_found=hidden_until_found
            class=classes("cn-accordion", class)
            attr:data-slot="accordion"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::AccordionRoot>
    }
}

#[component(element = "div")]
pub fn AccordionItem(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] value: Option<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, &'static str)>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::AccordionItem
            value=value
            disabled=disabled
            on_open_change=on_open_change
            class=classes("cn-accordion-item", class)
            attr:data-slot="accordion-item"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::AccordionItem>
    }
}

#[component(element = "button", skip(r#type, id))]
pub fn AccordionTrigger(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::AccordionHeader class="flex">
            <primitives::AccordionTrigger
                id=id
                disabled=disabled
                tabindex=tabindex
                class=classes("cn-accordion-trigger group/accordion-trigger", class)
                attr:data-slot="accordion-trigger"
                {..attrs}
            >
                {children.map(|children| children())}
                <Icon
                    icon=semantic::ACCORDION_CHEVRON_DOWN
                    class="cn-accordion-trigger-icon group-aria-expanded/accordion-trigger:hidden"
                    attr:data-slot="accordion-trigger-icon"
                />
                <Icon
                    icon=semantic::ACCORDION_CHEVRON_UP
                    class="cn-accordion-trigger-icon hidden group-aria-expanded/accordion-trigger:inline"
                    attr:data-slot="accordion-trigger-icon"
                />
            </primitives::AccordionTrigger>
        </primitives::AccordionHeader>
    }
}

#[component(element = "div", skip(id, role))]
pub fn AccordionContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] hidden_until_found: Option<Signal<bool>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::AccordionPanel
            id=id
            hidden_until_found=hidden_until_found
            class="cn-accordion-content"
            attr:data-slot="accordion-content"
            {..attrs}
        >
            <div class=classes("cn-accordion-content-inner", class)>
                {children.map(|children| children())}
            </div>
        </primitives::AccordionPanel>
    }
}
