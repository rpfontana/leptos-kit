use super::utils::classes;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};
use leptos_kit_macros::component;

#[component(element = "nav")]
pub fn Breadcrumb(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <nav
            class=classes("cn-breadcrumb", class)
            aria-label="breadcrumb"
            data-slot="breadcrumb"
            {..attrs}
        >
            {children.map(|children| children())}
        </nav>
    }
}

#[component(element = "ol")]
pub fn BreadcrumbList(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <ol class=classes("cn-breadcrumb-list", class) data-slot="breadcrumb-list" {..attrs}>
            {children.map(|children| children())}
        </ol>
    }
}

#[component(element = "li")]
pub fn BreadcrumbItem(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <li class=classes("cn-breadcrumb-item", class) data-slot="breadcrumb-item" {..attrs}>
            {children.map(|children| children())}
        </li>
    }
}

#[component(element = "a")]
pub fn BreadcrumbLink(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <a class=classes("cn-breadcrumb-link", class) data-slot="breadcrumb-link" {..attrs}>
            {children.map(|children| children())}
        </a>
    }
}

#[component(element = "span", skip(role))]
pub fn BreadcrumbPage(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <span
            class=classes("cn-breadcrumb-page", class)
            role="link"
            aria-disabled="true"
            aria-current="page"
            data-slot="breadcrumb-page"
            {..attrs}
        >
            {children.map(|children| children())}
        </span>
    }
}

#[component(element = "li", skip(role))]
pub fn BreadcrumbSeparator(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <li
            class=classes("cn-breadcrumb-separator", class)
            role="presentation"
            aria-hidden="true"
            data-slot="breadcrumb-separator"
            {..attrs}
        >
            {match children {
                Some(children) => Either::Left(children()),
                None => {
                    Either::Right(
                        view! { <Icon icon=semantic::BREADCRUMB_CHEVRON_RIGHT class="cn-rtl-flip" /> },
                    )
                }
            }}
        </li>
    }
}

#[component(element = "span", skip(role))]
pub fn BreadcrumbEllipsis(#[prop(optional, into)] class: Option<Signal<String>>) -> impl IntoView {
    view! {
        <span
            class=classes("cn-breadcrumb-ellipsis", class)
            role="presentation"
            aria-hidden="true"
            data-slot="breadcrumb-ellipsis"
            {..attrs}
        >
            <Icon icon=semantic::BREADCRUMB_MORE_HORIZONTAL />
            <span class="sr-only">"More"</span>
        </span>
    }
}
