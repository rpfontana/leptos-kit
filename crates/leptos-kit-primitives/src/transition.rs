use leptos::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TransitionStatus {
    #[default]
    Idle,
    Starting,
    Ending,
}

pub(crate) fn transition_attributes(
    status: RwSignal<TransitionStatus>,
) -> impl leptos::attr::Attribute {
    view! {
        <{..}
            data-starting-style=move || (status.get() == TransitionStatus::Starting).then_some("")
            data-ending-style=move || (status.get() == TransitionStatus::Ending).then_some("")
        />
    }
}
