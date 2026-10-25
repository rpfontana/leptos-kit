use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

primitives::keywords! {
    #[derive(Default)]
    pub enum CardSize {
        #[default]
        Default = "default",
        Sm = "sm",
    }
}

#[component(element = "div")]
pub fn Card(
    #[prop(into, default = CardSize::Default.into())] size: Signal<CardSize>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-card group/card", class) data-size=primitives::keyword(size) data-slot="card" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn CardHeader(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div
            class=classes("cn-card-header group/card-header", class)
            data-slot="card-header"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn CardTitle(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-card-title cn-font-heading", class) data-slot="card-title" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn CardDescription(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-card-description", class) data-slot="card-description" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn CardAction(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-card-action", class) data-slot="card-action" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn CardContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-card-content", class) data-slot="card-content" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn CardFooter(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-card-footer", class) data-slot="card-footer" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}
