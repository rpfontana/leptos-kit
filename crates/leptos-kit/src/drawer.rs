use super::button::{ButtonSize, ButtonVariant, button_look};
use super::utils::classes;
use leptos::context::Provider;
use leptos::prelude::*;
use leptos_kit::macros::component;
use leptos_kit::primitives::{self, ChangeReason};

pub use primitives::{DrawerSnapPoint, SwipeDirection};

#[derive(Clone, Copy)]
struct DrawerLook {
    has_snap_points: Signal<bool>,
    modal: Signal<bool>,
    show_swipe_handle: Signal<bool>,
    swipe_direction: Signal<SwipeDirection>,
}

#[leptos::component]
pub fn Drawer(
    #[prop(optional, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, ChangeReason)>>,
    #[prop(into, default = true.into())] modal: Signal<bool>,
    #[prop(optional, into)] disable_pointer_dismissal: Signal<bool>,
    #[prop(optional, into)] show_swipe_handle: Signal<bool>,
    #[prop(into, default = SwipeDirection::Down.into())] swipe_direction: Signal<SwipeDirection>,
    #[prop(optional, into)] snap_points: Signal<Vec<DrawerSnapPoint>>,
    #[prop(optional, into)] snap_point: Option<Signal<Option<DrawerSnapPoint>>>,
    #[prop(optional)] default_snap_point: Option<DrawerSnapPoint>,
    #[prop(optional, into)] on_snap_point_change: Option<
        Callback<(Option<DrawerSnapPoint>, ChangeReason)>,
    >,
    #[prop(optional, into)] snap_to_sequential_points: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let look = DrawerLook {
        has_snap_points: Signal::derive(move || snap_points.with(|points| !points.is_empty())),
        modal,
        show_swipe_handle,
        swipe_direction,
    };
    view! {
        <Provider value=look>
            <primitives::DrawerRoot
                nostrip:open=open
                default_open=default_open
                nostrip:on_open_change=on_open_change
                modal=modal
                disable_pointer_dismissal=disable_pointer_dismissal
                swipe_direction=swipe_direction
                snap_points=snap_points
                nostrip:snap_point=snap_point
                nostrip:default_snap_point=default_snap_point
                nostrip:on_snap_point_change=on_snap_point_change
                snap_to_sequential_points=snap_to_sequential_points
            >
                {children()}
            </primitives::DrawerRoot>
        </Provider>
    }
}

#[component(element = "button", skip(r#type))]
pub fn DrawerTrigger(
    #[prop(optional, into)] variant: Option<Signal<ButtonVariant>>,
    #[prop(optional, into)] size: Option<Signal<ButtonSize>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::DrawerTrigger
            disabled=disabled.unwrap_or_default()
            tabindex=tabindex.unwrap_or_else(|| Signal::stored(0))
            attr:data-slot="drawer-trigger"
            {..button_look(variant, size, class)}
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::DrawerTrigger>
    }
}

#[component(element = "button", skip(r#type))]
pub fn DrawerClose(
    #[prop(optional, into)] variant: Option<Signal<ButtonVariant>>,
    #[prop(optional, into)] size: Option<Signal<ButtonSize>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::DrawerClose
            disabled=disabled.unwrap_or_default()
            tabindex=tabindex.unwrap_or_else(|| Signal::stored(0))
            attr:data-slot="drawer-close"
            {..button_look(variant, size, class)}
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::DrawerClose>
    }
}

#[component(element = "dialog", skip(id, open, closedby, style, role))]
pub fn DrawerPortal(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <primitives::DrawerPortal attr:data-slot="drawer-portal" {..attrs}>
            {children.map(|children| children())}
        </primitives::DrawerPortal>
    }
}

#[component(element = "div", skip(role))]
pub fn DrawerOverlay(#[prop(optional, into)] class: Option<Signal<String>>) -> impl IntoView {
    view! {
        <primitives::DrawerBackdrop
            class=classes("cn-drawer-overlay", class)
            attr:data-slot="drawer-overlay"
            {..attrs}
        />
    }
}

#[component(element = "div")]
pub fn DrawerSwipeHandle(#[prop(optional, into)] class: Option<Signal<String>>) -> impl IntoView {
    view! {
        <div
            class=classes("cn-drawer-swipe-handle", class)
            data-slot="drawer-swipe-handle"
            aria-hidden="true"
            {..attrs}
        ></div>
    }
}

#[component(element = "div", skip(tabindex))]
pub fn DrawerContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let look = expect_context::<DrawerLook>();
    let snap_points = move || look.has_snap_points.get().then_some("");
    let swipe_axis = move || match look.swipe_direction.get() {
        SwipeDirection::Up | SwipeDirection::Down => "y",
        SwipeDirection::Left | SwipeDirection::Right => "x",
    };
    view! {
        <DrawerPortal>
            {move || look.modal.get().then(|| view! { <DrawerOverlay attr:data-snap-points=snap_points /> })}
            <primitives::DrawerViewport
                class="pointer-events-none fixed inset-0 z-50 select-none data-[modal=true]:pointer-events-auto"
                attr:data-slot="drawer-viewport"
                attr:data-modal=move || look.modal.get().to_string()
            >
                <primitives::DrawerPopup
                    class=classes("cn-drawer-popup group/drawer-popup", class)
                    attr:data-swipe-axis=swipe_axis
                    attr:data-snap-points=snap_points
                    attr:data-slot="drawer-popup"
                    {..attrs}
                >
                    {move || look.show_swipe_handle.get().then(|| view! { <DrawerSwipeHandle /> })}
                    <primitives::DrawerContent
                        class="cn-drawer-content-base"
                        attr:data-slot="drawer-content"
                    >
                        {children.map(|children| children())}
                    </primitives::DrawerContent>
                </primitives::DrawerPopup>
            </primitives::DrawerViewport>
        </DrawerPortal>
    }
}

#[component(element = "div")]
pub fn DrawerHeader(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-drawer-header-base", class) data-slot="drawer-header" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn DrawerFooter(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-drawer-footer-base", class) data-slot="drawer-footer" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

// the primitive makes up the id the dialog points at
#[component(element = "h2", skip(id))]
pub fn DrawerTitle(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::DrawerTitle
            class=classes("cn-drawer-title cn-font-heading", class)
            attr:data-slot="drawer-title"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::DrawerTitle>
    }
}

#[component(element = "p", skip(id))]
pub fn DrawerDescription(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::DrawerDescription
            class=classes("cn-drawer-description", class)
            attr:data-slot="drawer-description"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::DrawerDescription>
    }
}
