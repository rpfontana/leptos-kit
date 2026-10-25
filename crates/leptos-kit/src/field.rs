use super::separator::Separator;
use super::utils::classes;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

primitives::keywords! {
    #[derive(Default)]
    pub enum FieldLegendVariant {
        #[default]
        Legend = "legend",
        Label = "label",
    }
}

primitives::keywords! {
    #[derive(Default)]
    pub enum FieldOrientation {
        #[default]
        Vertical = "vertical",
        Horizontal = "horizontal",
        Responsive = "responsive",
    }
}

#[component(element = "fieldset")]
pub fn FieldSet(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <fieldset class=classes("cn-field-set", class) data-slot="field-set" {..attrs}>
            {children.map(|children| children())}
        </fieldset>
    }
}

#[component(element = "legend")]
pub fn FieldLegend(
    #[prop(into, default = FieldLegendVariant::Legend.into())] variant: Signal<FieldLegendVariant>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <legend
            class=classes("cn-field-legend", class)
            data-variant=primitives::keyword(variant)
            data-slot="field-legend"
            {..attrs}
        >
            {children.map(|children| children())}
        </legend>
    }
}

#[component(element = "div")]
pub fn FieldGroup(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div
            class=classes("cn-field-group group/field-group", class)
            data-slot="field-group"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div", skip(role))]
pub fn Field(
    #[prop(into, default = FieldOrientation::Vertical.into())] orientation: Signal<
        FieldOrientation,
    >,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        format!(
            "cn-field group/field cn-field-orientation-{}",
            orientation.get().as_str()
        )
    };
    view! {
        <div
            class=classes(own, class)
            role="group"
            data-orientation=primitives::keyword(orientation)
            data-slot="field"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn FieldContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div
            class=classes("cn-field-content group/field-content", class)
            data-slot="field-content"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

// upstream's `Label` with another slot, which a second `data-slot` can't override
#[component(element = "label")]
pub fn FieldLabel(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <label
            class=classes("cn-label cn-field-label group/field-label peer/field-label", class)
            data-slot="field-label"
            {..attrs}
        >
            {children.map(|children| children())}
        </label>
    }
}

#[component(element = "div")]
pub fn FieldTitle(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=classes("cn-field-title", class) data-slot="field-label" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "p")]
pub fn FieldDescription(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <p class=classes("cn-field-description", class) data-slot="field-description" {..attrs}>
            {children.map(|children| children())}
        </p>
    }
}

#[component(element = "div")]
pub fn FieldSeparator(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let content = children.is_some();
    view! {
        <div
            class=classes("cn-field-separator", class)
            data-content=content.to_string()
            data-slot="field-separator"
            {..attrs}
        >
            <Separator class="absolute inset-0 top-1/2" />
            {children
                .map(|children| {
                    view! {
                        <span class="cn-field-separator-content" data-slot="field-separator-content">
                            {children()}
                        </span>
                    }
                })}
        </div>
    }
}

#[component(element = "div", skip(role))]
pub fn FieldError(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] errors: Option<Signal<Vec<String>>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    match children {
        Some(children) => Either::Left(view! {
            <div class=classes("cn-field-error", class) role="alert" data-slot="field-error" {..attrs}>
                {children()}
            </div>
        }),
        None => Either::Right(move || {
            let content = messages(errors.map(|errors| errors.get()).unwrap_or_default())?;
            Some(view! {
                <div
                    class=classes("cn-field-error", class)
                    role="alert"
                    data-slot="field-error"
                    {..attrs.clone()}
                >
                    {content}
                </div>
            })
        }),
    }
}

// one message as text, several as a list, like upstream's dedupe by message
fn messages(errors: Vec<String>) -> Option<impl IntoView + use<>> {
    let mut unique: Vec<String> = Vec::new();
    for error in errors {
        if !unique.contains(&error) {
            unique.push(error);
        }
    }
    match unique.len() {
        0 => None,
        1 => (!unique[0].is_empty()).then(|| Either::Left(unique.remove(0))),
        _ => Some(Either::Right(view! {
            <ul class="ml-4 flex list-disc flex-col gap-1">
                {unique
                    .into_iter()
                    .filter(|message| !message.is_empty())
                    .map(|message| view! { <li>{message}</li> })
                    .collect_view()}
            </ul>
        })),
    }
}
