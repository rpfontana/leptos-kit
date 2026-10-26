use super::utils::classes;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
#[non_exhaustive]
pub enum ButtonVariant {
    #[default]
    Default,
    Outline,
    Secondary,
    Ghost,
    Destructive,
    Link,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
#[non_exhaustive]
pub enum ButtonSize {
    #[default]
    Default,
    Xs,
    Sm,
    Lg,
    Icon,
    IconXs,
    IconSm,
    IconLg,
}

#[component(element = "button")]
pub fn Button(
    #[prop(into, default = ButtonVariant::Default.into())] variant: Signal<ButtonVariant>,
    #[prop(into, default = ButtonSize::Default.into())] size: Signal<ButtonSize>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] focusable_when_disabled: Option<Signal<bool>>,
    #[prop(optional, into)] r#type: Option<Signal<String>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional, into)] href: Option<Signal<String>>,
    #[prop(optional, into)] target: Option<Signal<String>>,
    #[prop(optional, into)] rel: Option<Signal<String>>,
    #[prop(optional, into)] download: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let class = classes(move || button_classes(variant.get(), size.get()), class);
    // a link, so neither base ui's role="button" nor the disabled handling
    if let Some(href) = href {
        return Either::Left(view! {
            <a
                {..attrs}
                class=class
                href=href
                target=target
                rel=rel
                download=download
                tabindex=tabindex
                data-slot="button"
            >
                {children.map(|children| children())}
            </a>
        });
    }
    let disabled = move || disabled.is_some_and(|disabled| disabled.get());
    let focusable_when_disabled =
        move || focusable_when_disabled.is_some_and(|focusable| focusable.get());
    Either::Right(view! {
        <button
            {..attrs}
            {..primitives::button_attributes(disabled, focusable_when_disabled, tabindex)}
            class=class
            type=move || r#type.map_or_else(|| "button".to_string(), |r#type| r#type.get())
            data-slot="button"
        >
            {children.map(|children| children())}
        </button>
    })
}

pub(crate) fn button_classes(variant: ButtonVariant, size: ButtonSize) -> String {
    format!(
        "cn-button group/button cn-button-variant-{} cn-button-size-{}",
        variant.as_str(),
        size.as_str()
    )
}

// somehow like `render={<Button …/>}`
pub(crate) fn button_look(
    variant: Option<Signal<ButtonVariant>>,
    size: Option<Signal<ButtonSize>>,
    class: Option<Signal<String>>,
) -> impl leptos::attr::Attribute {
    let looks = variant.is_some() || size.is_some();
    if !looks && class.is_none() {
        return Either::Left(());
    }
    Either::Right(leptos::tachys::html::class::class(move || {
        let own = looks.then(|| {
            button_classes(
                variant.map_or(ButtonVariant::Default, |variant| variant.get()),
                size.map_or(ButtonSize::Default, |size| size.get()),
            )
        });
        [own, class.map(|class| class.get())]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ")
    }))
}
