use crate::classes;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[component(element = "button")]
pub fn Button(
    #[prop(default = "default")] variant: &'static str,
    #[prop(default = "default")] size: &'static str,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional)] focusable_when_disabled: bool,
    #[prop(optional, into)] r#type: Option<Signal<String>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::Button
            class=classes(button_classes(variant, size), class)
            disabled=disabled.unwrap_or_default()
            focusable_when_disabled
            r#type=r#type.unwrap_or_else(|| Signal::stored("button".into()))
            tabindex=tabindex.unwrap_or_else(|| Signal::stored(0))
            data-slot="button"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::Button>
    }
}

pub(crate) fn button_classes(variant: &str, size: &str) -> String {
    #[cfg(debug_assertions)]
    check(variant, size);
    format!("cn-button group/button cn-button-variant-{variant} cn-button-size-{size}")
}

// somehow like `render={<Button …/>}`
pub(crate) fn button_look(
    variant: Option<&'static str>,
    size: Option<&'static str>,
    class: Option<Signal<String>>,
) -> impl leptos::attr::Attribute {
    let own = (variant.is_some() || size.is_some())
        .then(|| button_classes(variant.unwrap_or("default"), size.unwrap_or("default")));
    if own.is_none() && class.is_none() {
        return Either::Left(());
    }
    Either::Right(leptos::tachys::html::class::class(move || {
        [own.clone(), class.map(|class| class.get())]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ")
    }))
}

#[cfg(debug_assertions)]
fn check(variant: &str, size: &str) {
    const VARIANTS: &[&str] = &[
        "default",
        "outline",
        "secondary",
        "ghost",
        "destructive",
        "link",
    ];
    const SIZES: &[&str] = &[
        "default", "xs", "sm", "lg", "icon", "icon-xs", "icon-sm", "icon-lg",
    ];
    if !VARIANTS.contains(&variant) {
        leptos::logging::warn!("Button: unknown variant `{variant}`, expected one of {VARIANTS:?}");
    }
    if !SIZES.contains(&size) {
        leptos::logging::warn!("Button: unknown size `{size}`, expected one of {SIZES:?}");
    }
}
