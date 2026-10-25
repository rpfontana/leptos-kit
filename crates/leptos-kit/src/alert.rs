use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

primitives::keywords! {
    #[derive(Default)]
    pub enum AlertVariant {
        #[default]
        Default = "default",
        Destructive = "destructive",
    }
}

#[component(element = "div", skip(role))]
pub fn Alert(
    #[prop(into, default = AlertVariant::Default.into())] variant: Signal<AlertVariant>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        format!(
            "cn-alert group/alert cn-alert-variant-{}",
            variant.get().as_str()
        )
    };
    view! {
        <div
            class=classes(own, class)
            data-slot="alert"
            role="alert"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn AlertTitle(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-alert-title", class) data-slot="alert-title" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn AlertDescription(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-alert-description", class) data-slot="alert-description" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn AlertAction(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-alert-action", class) data-slot="alert-action" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}
