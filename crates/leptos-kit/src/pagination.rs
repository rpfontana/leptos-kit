use super::button::{ButtonSize, ButtonVariant, button_classes};
use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};
use leptos_kit_macros::component;

#[component(element = "nav", skip(role))]
pub fn Pagination(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <nav
            class=classes("cn-pagination", class)
            role="navigation"
            aria-label="pagination"
            data-slot="pagination"
            {..attrs}
        >
            {children.map(|children| children())}
        </nav>
    }
}

#[component(element = "ul")]
pub fn PaginationContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <ul class=classes("cn-pagination-content", class) data-slot="pagination-content" {..attrs}>
            {children.map(|children| children())}
        </ul>
    }
}

#[component(element = "li")]
pub fn PaginationItem(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <li data-slot="pagination-item" {..attrs}>
            {children.map(|children| children())}
        </li>
    }
}

#[component(element = "a")]
pub fn PaginationLink(
    #[prop(optional, into)] is_active: Signal<bool>,
    #[prop(into, default = ButtonSize::Icon.into())] size: Signal<ButtonSize>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        let variant = if is_active.get() {
            ButtonVariant::Outline
        } else {
            ButtonVariant::Ghost
        };
        format!("{} cn-pagination-link", button_classes(variant, size.get()))
    };
    view! {
        <a
            class=classes(own, class)
            aria-current=move || is_active.get().then_some("page")
            data-active=move || is_active.get().then_some("true")
            data-slot="pagination-link"
            {..attrs}
        >
            {children.map(|children| children())}
        </a>
    }
}

#[component(element = "a")]
pub fn PaginationPrevious(
    #[prop(optional, into)] is_active: Signal<bool>,
    #[prop(into, default = ButtonSize::Default.into())] size: Signal<ButtonSize>,
    #[prop(into, default = "Previous".into())] text: Signal<String>,
    #[prop(optional, into)] class: Option<Signal<String>>,
) -> impl IntoView {
    view! {
        <PaginationLink
            is_active=is_active
            size=size
            class=Signal::derive(classes("cn-pagination-previous", class))
            {..attrs}
            attr:aria-label="Go to previous page"
        >
            <Icon
                icon=semantic::PAGINATION_CHEVRON_LEFT
                class="cn-rtl-flip"
                attr:data-icon="inline-start"
            />
            <span class="cn-pagination-previous-text">{text}</span>
        </PaginationLink>
    }
}

#[component(element = "a")]
pub fn PaginationNext(
    #[prop(optional, into)] is_active: Signal<bool>,
    #[prop(into, default = ButtonSize::Default.into())] size: Signal<ButtonSize>,
    #[prop(into, default = "Next".into())] text: Signal<String>,
    #[prop(optional, into)] class: Option<Signal<String>>,
) -> impl IntoView {
    view! {
        <PaginationLink
            is_active=is_active
            size=size
            class=Signal::derive(classes("cn-pagination-next", class))
            {..attrs}
            attr:aria-label="Go to next page"
        >
            <span class="cn-pagination-next-text">{text}</span>
            <Icon
                icon=semantic::PAGINATION_CHEVRON_RIGHT
                class="cn-rtl-flip"
                attr:data-icon="inline-end"
            />
        </PaginationLink>
    }
}

#[component(element = "span")]
pub fn PaginationEllipsis(#[prop(optional, into)] class: Option<Signal<String>>) -> impl IntoView {
    view! {
        <span
            class=classes("cn-pagination-ellipsis", class)
            aria-hidden="true"
            data-slot="pagination-ellipsis"
            {..attrs}
        >
            <Icon icon=semantic::PAGINATION_MORE_HORIZONTAL />
            <span class="sr-only">"More pages"</span>
        </span>
    }
}
