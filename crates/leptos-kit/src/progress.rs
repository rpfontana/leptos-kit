use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[component(element = "div", skip(role))]
pub fn Progress(
    #[prop(optional, into)] value: Signal<Option<f64>>,
    #[prop(into, default = 0.0.into())] min: Signal<f64>,
    #[prop(into, default = 100.0.into())] max: Signal<f64>,
    #[prop(optional, into)] format: Option<Callback<f64, String>>,
    #[prop(optional, into)] get_aria_value_text: Option<Callback<(String, Option<f64>), String>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ProgressRoot
            value=value
            min=min
            max=max
            format=format
            get_aria_value_text=get_aria_value_text
            class=classes("cn-progress-root", class)
            attr:data-slot="progress"
            {..attrs}
        >
            {children.map(|children| children())}
            <ProgressTrack>
                <ProgressIndicator />
            </ProgressTrack>
        </primitives::ProgressRoot>
    }
}

#[component(element = "div")]
pub fn ProgressTrack(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ProgressTrack
            class=classes("cn-progress-track", class)
            attr:data-slot="progress-track"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::ProgressTrack>
    }
}

#[component(element = "div")]
pub fn ProgressIndicator(#[prop(optional, into)] class: Option<Signal<String>>) -> impl IntoView {
    view! {
        <primitives::ProgressIndicator
            class=classes("cn-progress-indicator", class)
            attr:data-slot="progress-indicator"
            {..attrs}
        />
    }
}

#[component(element = "span", skip(id, role))]
pub fn ProgressLabel(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::ProgressLabel
            class=classes("cn-progress-label", class)
            attr:data-slot="progress-label"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::ProgressLabel>
    }
}

#[component(element = "span")]
pub fn ProgressValue(#[prop(optional, into)] class: Option<Signal<String>>) -> impl IntoView {
    view! {
        <primitives::ProgressValue
            class=classes("cn-progress-value", class)
            attr:data-slot="progress-value"
            {..attrs}
        />
    }
}
