use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
#[non_exhaustive]
pub enum AvatarSize {
    #[default]
    Default,
    Sm,
    Lg,
}

#[component(element = "span")]
pub fn Avatar(
    #[prop(into, default = AvatarSize::Default.into())] size: Signal<AvatarSize>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::AvatarRoot
            class=classes("cn-avatar group/avatar", class)
            attr:data-size=primitives::keyword(size)
            attr:data-slot="avatar"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::AvatarRoot>
    }
}

#[component(element = "img")]
pub fn AvatarImage(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] src: Option<Signal<String>>,
    #[prop(optional, into)] keep_mounted: Signal<bool>,
    #[prop(optional, into)] on_loading_status_change: Option<
        Callback<primitives::ImageLoadingStatus>,
    >,
) -> impl IntoView {
    view! {
        <primitives::AvatarImage
            src=src
            keep_mounted=keep_mounted
            on_loading_status_change=on_loading_status_change
            class=classes("cn-avatar-image", class)
            attr:data-slot="avatar-image"
            {..attrs}
        />
    }
}

#[component(element = "span")]
pub fn AvatarFallback(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] delay: Signal<u32>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::AvatarFallback
            delay=delay
            class=classes("cn-avatar-fallback", class)
            attr:data-slot="avatar-fallback"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::AvatarFallback>
    }
}

#[component(element = "span")]
pub fn AvatarBadge(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <span class=classes("cn-avatar-badge", class) data-slot="avatar-badge" {..attrs}>
            {children.map(|children| children())}
        </span>
    }
}

#[component(element = "div")]
pub fn AvatarGroup(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div
            class=classes("cn-avatar-group group/avatar-group", class)
            data-slot="avatar-group"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn AvatarGroupCount(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-avatar-group-count", class) data-slot="avatar-group-count" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}
