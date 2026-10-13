use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;

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
    #[prop(into, default = "default".into())] variant: Signal<&'static str>,
    #[prop(into, default = "start".into())] align: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        #[cfg(debug_assertions)]
        check("Bubble", "variant", variant.get(), VARIANTS);
        format!("cn-bubble group/bubble cn-bubble-variant-{}", variant.get())
    };
    let align = move || {
        #[cfg(debug_assertions)]
        check("Bubble", "align", align.get(), ALIGNS);
        align.get()
    };
    view! {
        <div
            class=classes(own, class)
            data-variant=variant
            data-align=align
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
    #[prop(into, default = "bottom".into())] side: Signal<&'static str>,
    #[prop(into, default = "end".into())] align: Signal<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        #[cfg(debug_assertions)]
        {
            check("BubbleReactions", "side", side.get(), SIDES);
            check("BubbleReactions", "align", align.get(), ALIGNS);
        }
        format!(
            "cn-bubble-reactions cn-bubble-reactions-side-{} cn-bubble-reactions-align-{}",
            side.get(),
            align.get()
        )
    };
    view! {
        <div
            class=classes(own, class)
            data-align=align
            data-side=side
            data-slot="bubble-reactions"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[cfg(debug_assertions)]
const VARIANTS: &[&str] = &[
    "default",
    "secondary",
    "muted",
    "tinted",
    "outline",
    "ghost",
    "destructive",
];
#[cfg(debug_assertions)]
const ALIGNS: &[&str] = &["start", "end"];
#[cfg(debug_assertions)]
const SIDES: &[&str] = &["top", "bottom"];

#[cfg(debug_assertions)]
fn check(part: &str, prop: &str, value: &str, known: &[&str]) {
    if !known.contains(&value) {
        leptos::logging::warn!("{part}: unknown {prop} `{value}`, expected one of {known:?}");
    }
}
