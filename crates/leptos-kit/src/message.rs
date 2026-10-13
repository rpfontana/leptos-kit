use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "div")]
pub fn MessageGroup(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-message-group", class) data-slot="message-group" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn Message(
    #[prop(into, default = "start".into())] align: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let align = move || {
        #[cfg(debug_assertions)]
        check(align.get());
        align.get()
    };
    view! {
        <div
            class=classes("cn-message group/message", class)
            data-align=align
            data-slot="message"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn MessageAvatar(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-message-avatar", class) data-slot="message-avatar" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn MessageContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-message-content", class) data-slot="message-content" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn MessageHeader(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-message-header", class) data-slot="message-header" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn MessageFooter(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-message-footer", class) data-slot="message-footer" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[cfg(debug_assertions)]
fn check(align: &str) {
    const ALIGNS: &[&str] = &["start", "end"];
    if !ALIGNS.contains(&align) {
        leptos::logging::warn!("Message: unknown align `{align}`, expected one of {ALIGNS:?}");
    }
}
