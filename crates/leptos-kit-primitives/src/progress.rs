use crate::id::ElementId;
use crate::{Keyword, use_id};
use leptos::context::Provider;
use leptos::html;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Keyword)]
enum ProgressStatus {
    Indeterminate,
    Progressing,
    Complete,
}

#[derive(Clone, Copy)]
struct ProgressContext {
    status: Memo<ProgressStatus>,
    percentage: Memo<Option<f64>>,
    formatted: Memo<String>,
    label: RwSignal<Option<ElementId<html::Span>>>,
}

#[component(element = "div", skip(role))]
pub fn ProgressRoot(
    #[prop(optional, into)] value: Signal<Option<f64>>,
    #[prop(into, default = 0.0.into())] min: Signal<f64>,
    #[prop(into, default = 100.0.into())] max: Signal<f64>,
    #[prop(optional_no_strip, into)] format: Option<Callback<f64, String>>,
    #[prop(optional_no_strip, into)] get_aria_value_text: Option<
        Callback<(String, Option<f64>), String>,
    >,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let clamped = Memo::new(move |_| {
        let value = value.get().filter(|value| value.is_finite())?;
        Some(value.max(min.get()).min(max.get()))
    });
    let percentage = Memo::new(move |_| {
        let value = value.get().filter(|value| value.is_finite())?;
        let percent = (value - min.get()) * 100.0 / (max.get() - min.get());
        Some(if percent.is_nan() {
            0.0
        } else {
            percent.clamp(0.0, 100.0)
        })
    });
    let status = Memo::new(move |_| match clamped.get() {
        None => ProgressStatus::Indeterminate,
        Some(value) if value == max.get() => ProgressStatus::Complete,
        Some(_) => ProgressStatus::Progressing,
    });
    // base ui formats with intl, a percentage rounds the same way
    let formatted = Memo::new(move |_| match (clamped.get(), format) {
        (Some(value), Some(format)) => format.run(value),
        (Some(_), None) => format!("{}%", percentage.get().unwrap_or(0.0).round()),
        (None, _) => String::new(),
    });
    let context = ProgressContext {
        status,
        percentage,
        formatted,
        label: RwSignal::new(None),
    };

    let text = move || match get_aria_value_text {
        Some(text) => text.run((formatted.get(), value.get())),
        None if clamped.get().is_none() => String::from("indeterminate progress"),
        None => formatted.get(),
    };
    view! {
        <div
            {..attrs}
            {..status_attributes(context)}
            role="progressbar"
            aria-labelledby=move || context.label.get().map(|label| label.get())
            aria-valuemax=move || max.get().to_string()
            aria-valuemin=move || min.get().to_string()
            aria-valuenow=move || clamped.get().map(|value| value.to_string())
            aria-valuetext=text
        >
            // components share their parent's owner, a sibling root would overwrite the context
            <Provider value=context>{children.map(|children| children())}</Provider>
            // makes nvda read the label, as in base ui
            <span
                role="presentation"
                style="clip-path: inset(50%); overflow: hidden; white-space: nowrap; border: 0; padding: 0; width: 1px; height: 1px; margin: -1px; position: fixed; top: 0; left: 0;"
            >
                "x"
            </span>
        </div>
    }
}

#[component(element = "div")]
pub fn ProgressTrack(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<ProgressContext>();
    view! {
        <div {..attrs} {..status_attributes(context)}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn ProgressIndicator(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<ProgressContext>();
    // indeterminate leaves the width to the style
    let fill = move |value: &'static str| move || context.percentage.get().map(|_| value);
    let width = move || {
        context
            .percentage
            .get()
            .map(|percent| format!("{percent}%"))
    };
    view! {
        <div
            {..attrs}
            {..status_attributes(context)}
            style:inset-inline-start=fill("0")
            style:height=fill("inherit")
            style:width=width
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "span", skip(id, role))]
pub fn ProgressLabel(
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<ProgressContext>();
    let span = NodeRef::new();
    let label = ElementId::new(span, id.unwrap_or_else(use_id));
    context.label.set(Some(label));
    on_cleanup(move || {
        if context.label.get_untracked() == Some(label) {
            context.label.set(None);
        }
    });
    view! {
        <span
            {..attrs}
            {..status_attributes(context)}
            node_ref=span
            id=label.get()
            role="presentation"
        >
            {children.map(|children| children())}
        </span>
    }
}

#[component(element = "span")]
pub fn ProgressValue() -> impl IntoView {
    let context = expect_context::<ProgressContext>();
    view! {
        <span {..attrs} {..status_attributes(context)} aria-hidden="true">
            {move || context.formatted.get()}
        </span>
    }
}

fn status_attributes(context: ProgressContext) -> impl leptos::attr::Attribute {
    let is = move |status: ProgressStatus| move || (context.status.get() == status).then_some("");
    view! {
        <{..}
            data-progressing=is(ProgressStatus::Progressing)
            data-complete=is(ProgressStatus::Complete)
            data-indeterminate=is(ProgressStatus::Indeterminate)
        />
    }
}
