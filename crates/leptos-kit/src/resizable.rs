use super::utils::classes;
use leptos::prelude::*;
use leptos_kit::macros::component;
use leptos_kit::primitives::{self, Orientation};

pub use primitives::{
    GroupRef, GroupResizeBehavior, Layout, LayoutChangedMeta, Length, PanelRef, PanelSize,
};

#[component(element = "div")]
pub fn ResizablePanelGroup(
    #[prop(into, default = Orientation::Horizontal.into())] orientation: Signal<Orientation>,
    #[prop(optional)] default_layout: Option<Layout>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] disable_cursor: Signal<bool>,
    #[prop(optional, into)] on_layout_change: Option<Callback<Layout>>,
    #[prop(optional, into)] on_layout_changed: Option<Callback<(Layout, LayoutChangedMeta)>>,
    #[prop(optional)] group_ref: Option<GroupRef>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ResizableGroup
            orientation=orientation
            default_layout=default_layout
            disabled=disabled
            disable_cursor=disable_cursor
            on_layout_change=on_layout_change
            on_layout_changed=on_layout_changed
            group_ref=group_ref
            class=classes("cn-resizable-panel-group", class)
            attr:data-slot="resizable-panel-group"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::ResizableGroup>
    }
}

#[component(element = "div")]
pub fn ResizablePanel(
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] default_size: Option<Length>,
    #[prop(optional, into)] min_size: Option<Length>,
    #[prop(optional, into)] max_size: Option<Length>,
    #[prop(optional, into)] collapsed_size: Option<Length>,
    #[prop(optional)] collapsible: bool,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] group_resize_behavior: GroupResizeBehavior,
    #[prop(optional, into)] on_resize: Option<Callback<(PanelSize, Option<PanelSize>)>>,
    #[prop(optional)] panel_ref: Option<PanelRef>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] style: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ResizablePanel
            id=id
            default_size=default_size
            min_size=min_size
            max_size=max_size
            collapsed_size=collapsed_size
            collapsible=collapsible
            disabled=disabled
            group_resize_behavior=group_resize_behavior
            on_resize=on_resize
            panel_ref=panel_ref
            class=class
            style=style
            attr:data-slot="resizable-panel"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::ResizablePanel>
    }
}

#[component(element = "div", skip(role, tabindex))]
pub fn ResizableHandle(
    #[prop(optional)] with_handle: bool,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] disable_double_click: bool,
    #[prop(optional, into)] class: Option<Signal<String>>,
) -> impl IntoView {
    view! {
        <primitives::ResizableSeparator
            disabled=disabled
            disable_double_click=disable_double_click
            class=classes("cn-resizable-handle", class)
            attr:data-slot="resizable-handle"
            {..attrs}
        >
            {with_handle.then(|| view! { <div class="cn-resizable-handle-icon"></div> })}
        </primitives::ResizableSeparator>
    }
}
