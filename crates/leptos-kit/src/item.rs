use super::utils::classes;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
#[non_exhaustive]
pub enum ItemVariant {
    #[default]
    Default,
    Outline,
    Muted,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
#[non_exhaustive]
pub enum ItemSize {
    #[default]
    Default,
    Sm,
    Xs,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
#[non_exhaustive]
pub enum ItemMediaVariant {
    #[default]
    Default,
    Icon,
    Image,
}

#[component(element = "div", skip(role))]
pub fn ItemGroup(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div
            class=classes("cn-item-group group/item-group", class)
            role="list"
            data-slot="item-group"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div", skip(role))]
pub fn ItemSeparator(
    #[prop(into, default = primitives::Orientation::Horizontal.into())] orientation: Signal<
        primitives::Orientation,
    >,
    #[prop(optional, into)] class: Option<Signal<String>>,
) -> impl IntoView {
    view! {
        <div
            class=classes("cn-item-separator", class)
            role="separator"
            aria-orientation=primitives::keyword(orientation)
            data-orientation=primitives::keyword(orientation)
            data-slot="item-separator"
            {..attrs}
        />
    }
}

#[component(element = "div")]
pub fn Item(
    #[prop(into, default = ItemVariant::Default.into())] variant: Signal<ItemVariant>,
    #[prop(into, default = ItemSize::Default.into())] size: Signal<ItemSize>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] href: Option<Signal<String>>,
    #[prop(optional, into)] target: Option<Signal<String>>,
    #[prop(optional, into)] rel: Option<Signal<String>>,
    #[prop(optional, into)] download: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        format!(
            "cn-item group/item cn-item-variant-{} cn-item-size-{}",
            variant.get().as_str(),
            size.get().as_str()
        )
    };
    let class = classes(own, class);
    match href {
        Some(href) => Either::Left(view! {
            <a
                class=class
                href=href
                target=target
                rel=rel
                download=download
                data-variant=primitives::keyword(variant)
                data-size=primitives::keyword(size)
                data-slot="item"
                {..attrs}
            >
                {children.map(|children| children())}
            </a>
        }),
        None => Either::Right(view! {
            <div
                class=class
                data-variant=primitives::keyword(variant)
                data-size=primitives::keyword(size)
                data-slot="item"
                {..attrs}
            >
                {children.map(|children| children())}
            </div>
        }),
    }
}

#[component(element = "div")]
pub fn ItemMedia(
    #[prop(into, default = ItemMediaVariant::Default.into())] variant: Signal<ItemMediaVariant>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        format!(
            "cn-item-media cn-item-media-variant-{}",
            variant.get().as_str()
        )
    };
    view! {
        <div class=classes(own, class) data-variant=primitives::keyword(variant) data-slot="item-media" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn ItemContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-item-content", class) data-slot="item-content" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn ItemTitle(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-item-title", class) data-slot="item-title" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "p")]
pub fn ItemDescription(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <p class=classes("cn-item-description", class) data-slot="item-description" {..attrs}>
            {children.map(|children| children())}
        </p>
    }
}

#[component(element = "div")]
pub fn ItemActions(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-item-actions", class) data-slot="item-actions" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn ItemHeader(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-item-header", class) data-slot="item-header" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn ItemFooter(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-item-footer", class) data-slot="item-footer" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}
