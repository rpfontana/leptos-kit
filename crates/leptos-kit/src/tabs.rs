use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
#[non_exhaustive]
pub enum TabsListVariant {
    #[default]
    Default,
    Line,
}

#[component(element = "div")]
pub fn Tabs(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] value: Option<Signal<Option<String>>>,
    #[prop(optional, into)] default_value: Option<String>,
    #[prop(optional, into)] on_value_change: Option<Callback<(String, primitives::ChangeReason)>>,
    #[prop(into, default = primitives::Orientation::Horizontal.into())] orientation: Signal<
        primitives::Orientation,
    >,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::TabsRoot
            value=value
            default_value=default_value
            on_value_change=on_value_change
            orientation=orientation
            class=classes("cn-tabs group/tabs", class)
            attr:data-slot="tabs"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::TabsRoot>
    }
}

#[component(element = "div", skip(role))]
pub fn TabsList(
    #[prop(into, default = TabsListVariant::Default.into())] variant: Signal<TabsListVariant>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] activate_on_focus: Signal<bool>,
    #[prop(into, default = true.into())] loop_focus: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        let variant = variant.get().as_str();
        format!("cn-tabs-list group/tabs-list cn-tabs-list-variant-{variant}")
    };
    view! {
        <primitives::TabsList
            activate_on_focus=activate_on_focus
            loop_focus=loop_focus
            class=classes(own, class)
            attr:data-slot="tabs-list"
            attr:data-variant=move || variant.get().as_str()
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::TabsList>
    }
}

#[component(element = "button", skip(r#type, role, id, tabindex))]
pub fn TabsTrigger(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into)] value: String,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::TabsTab
            value=value
            disabled=disabled
            id=id
            class=classes("cn-tabs-trigger", class)
            attr:data-slot="tabs-trigger"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::TabsTab>
    }
}

#[component(element = "div", skip(id, role, tabindex))]
pub fn TabsContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into)] value: String,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::TabsPanel
            value=value
            id=id
            class=classes("cn-tabs-content", class)
            attr:data-slot="tabs-content"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::TabsPanel>
    }
}
