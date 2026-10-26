use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
#[non_exhaustive]
pub enum BubbleVariant {
    #[default]
    Default,
    Secondary,
    Muted,
    Tinted,
    Outline,
    Ghost,
    Destructive,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
pub enum BubbleAlign {
    #[default]
    Start,
    End,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
pub enum BubbleReactionsSide {
    #[default]
    Bottom,
    Top,
}

#[component(element = "div")]
pub fn BubbleGroup(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-bubble-group", class) data-slot="bubble-group" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn Bubble(
    #[prop(into, default = BubbleVariant::Default.into())] variant: Signal<BubbleVariant>,
    #[prop(into, default = BubbleAlign::Start.into())] align: Signal<BubbleAlign>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        format!(
            "cn-bubble group/bubble cn-bubble-variant-{}",
            variant.get().as_str()
        )
    };
    view! {
        <div
            class=classes(own, class)
            data-variant=primitives::keyword(variant)
            data-align=primitives::keyword(align)
            data-slot="bubble"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn BubbleContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-bubble-content", class) data-slot="bubble-content" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn BubbleReactions(
    #[prop(into, default = BubbleReactionsSide::Bottom.into())] side: Signal<BubbleReactionsSide>,
    #[prop(into, default = BubbleAlign::End.into())] align: Signal<BubbleAlign>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        format!(
            "cn-bubble-reactions cn-bubble-reactions-side-{} cn-bubble-reactions-align-{}",
            side.get().as_str(),
            align.get().as_str()
        )
    };
    view! {
        <div
            class=classes(own, class)
            data-align=primitives::keyword(align)
            data-side=primitives::keyword(side)
            data-slot="bubble-reactions"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}
