use super::button::{ButtonSize, ButtonVariant, button_classes};
use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;
use std::sync::LazyLock;

pub use primitives::{
    SwipeDirection, ToastActionOptions, ToastManager, ToastObject, ToastOptions, ToastPortal,
    ToastPriority, ToastProvider, UseToastManager, use_toast_manager,
};

static TOAST: LazyLock<ToastManager> = LazyLock::new(ToastManager::new);

// the manager a `Toaster` shows unless it's given another
pub fn toast() -> &'static ToastManager {
    &TOAST
}

pub fn create_toast_manager() -> ToastManager {
    ToastManager::new()
}

#[component(element = "div", skip(tabindex, role))]
pub fn ToastViewport(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ToastViewport
            class=classes(
                "pointer-events-none fixed inset-x-4 bottom-4 z-50 mx-auto w-auto max-w-sm outline-none sm:right-4 sm:left-auto sm:mx-0 sm:w-full",
                class,
            )
            attr:data-slot="toast-viewport"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::ToastViewport>
    }
}

#[component(element = "div", skip(role, tabindex))]
pub fn Toast(
    toast: ToastObject,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(default = vec![SwipeDirection::Down, SwipeDirection::Right])] swipe_direction: Vec<
        SwipeDirection,
    >,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ToastRoot
            toast=toast
            swipe_direction=swipe_direction
            class=classes("cn-toast group/toast", class)
            attr:data-slot="toast"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::ToastRoot>
    }
}

#[component(element = "div")]
pub fn ToastContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ToastContent
            class=classes(
                "flex h-full items-center gap-3 overflow-hidden p-4 transition-opacity duration-250 ease-[cubic-bezier(0.22,1,0.36,1)] data-behind:opacity-0 data-expanded:opacity-100",
                class,
            )
            attr:data-slot="toast-content"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::ToastContent>
    }
}

// without children the title, description and action show the toast's own
#[component(element = "h2", skip(id))]
pub fn ToastTitle(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ToastTitle
            nostrip:children=children
            class=classes("text-sm font-medium", class)
            attr:data-slot="toast-title"
            {..attrs}
        />
    }
}

#[component(element = "p", skip(id))]
pub fn ToastDescription(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ToastDescription
            nostrip:children=children
            class=classes("text-sm text-muted-foreground", class)
            attr:data-slot="toast-description"
            {..attrs}
        />
    }
}

#[component(element = "button", skip(r#type))]
pub fn ToastAction(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let look = button_classes(ButtonVariant::Outline, ButtonSize::Sm);
    view! {
        <primitives::ToastAction
            disabled=disabled
            nostrip:tabindex=tabindex
            nostrip:children=children
            class=classes(move || format!("{look} shrink-0"), class)
            attr:data-slot="toast-action"
            {..attrs}
        />
    }
}

#[component(element = "button", skip(r#type))]
pub fn ToastClose(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let look = button_classes(ButtonVariant::Ghost, ButtonSize::IconSm);
    view! {
        <primitives::ToastClose
            disabled=disabled
            nostrip:tabindex=tabindex
            class=classes(
                move || {
                    format!(
                        "{look} relative shrink-0 text-muted-foreground after:absolute after:-inset-2 after:content-[''] hover:text-foreground",
                    )
                },
                class,
            )
            attr:data-slot="toast-close"
            attr:aria-label="Close toast"
            {..attrs}
        >
            {match children {
                Some(children) => children().into_any(),
                None => view! { <Icon icon=semantic::TOAST_X /> }.into_any(),
            }}
        </primitives::ToastClose>
    }
}

fn toast_icon(kind: Option<&str>) -> Option<impl IntoView + use<>> {
    let (icon, class) = match kind? {
        "success" => (semantic::TOAST_CIRCLE_CHECK, None),
        "info" => (semantic::TOAST_INFO, None),
        "warning" => (semantic::TOAST_TRIANGLE_ALERT, None),
        "error" => (semantic::TOAST_OCTAGON_X, Some("text-destructive")),
        "loading" => (semantic::TOAST_LOADER_2, Some("animate-spin")),
        _ => return None,
    };
    Some(view! {
        <span
            data-slot="toast-icon"
            class="shrink-0 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4"
        >
            <Icon icon=icon nostrip:class=class.map(|class| Signal::stored(class.to_string())) />
        </span>
    })
}

#[leptos::component]
fn ToastList() -> impl IntoView {
    let toasts = use_toast_manager().toasts;
    view! {
        <For
            each=move || toasts.get()
            key=|toast| toast.id().to_string()
            children=move |toast: ToastObject| {
                let kind = toast.clone();
                view! {
                    <Toast toast=toast>
                        <ToastContent>
                            {move || toast_icon(kind.r#type().as_deref())}
                            <div class="flex min-w-0 flex-1 flex-col gap-1">
                                <ToastTitle />
                                <ToastDescription />
                            </div>
                            <ToastAction />
                            <ToastClose />
                        </ToastContent>
                    </Toast>
                }
            }
        />
    }
}

#[leptos::component]
pub fn Toaster(
    #[prop(into, default = 5000.into())] timeout: Signal<u32>,
    #[prop(into, default = 3.into())] limit: Signal<usize>,
    #[prop(optional)] toast_manager: Option<ToastManager>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <ToastProvider
            timeout=timeout
            limit=limit
            toast_manager=toast_manager.unwrap_or_else(|| toast().clone())
        >
            {children.map(|children| children())}
            <ToastPortal>
                <ToastViewport>
                    <ToastList />
                </ToastViewport>
            </ToastPortal>
        </ToastProvider>
    }
}
