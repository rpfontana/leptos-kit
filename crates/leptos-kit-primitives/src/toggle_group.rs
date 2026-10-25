use crate::composite::Composite;
use crate::{ChangeReason, Controlled, Orientation, keyword};
use leptos::context::Provider;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[derive(Clone, Copy)]
pub(crate) struct ToggleGroupContext {
    pub(crate) value: Controlled<Vec<String>>,
    pub(crate) disabled: Signal<bool>,
    pub(crate) composite: Composite,
    multiple: Signal<bool>,
}

impl ToggleGroupContext {
    pub(crate) fn change(&self, item: &str, pressed: bool, reason: ChangeReason) {
        let mut next = self.value.get_untracked();
        if !self.multiple.get_untracked() {
            next.clear();
        }
        if pressed {
            next.push(item.to_string());
        } else {
            next.retain(|value| value != item);
        }
        self.value.set(next, reason);
    }
}

#[component(element = "div", skip(role))]
pub fn ToggleGroup(
    #[prop(optional_no_strip, into)] value: Option<Signal<Vec<String>>>,
    #[prop(optional)] default_value: Vec<String>,
    #[prop(optional_no_strip, into)] on_value_change: Option<Callback<(Vec<String>, ChangeReason)>>,
    #[prop(optional, into)] multiple: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(into, default = Orientation::Horizontal.into())] orientation: Signal<Orientation>,
    #[prop(into, default = true.into())] loop_focus: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let composite = Composite::new(orientation, loop_focus).home_end();
    let context = ToggleGroupContext {
        value: Controlled::new(value, default_value, on_value_change),
        disabled,
        composite,
        multiple,
    };
    view! {
        <div
            {..attrs}
            {..composite.root_attributes()}
            role="group"
            data-disabled=move || disabled.get().then_some("")
            data-multiple=move || multiple.get().then_some("")
            data-orientation=keyword(orientation)
        >
            <Provider value=context>{children.map(|children| children())}</Provider>
        </div>
    }
}
