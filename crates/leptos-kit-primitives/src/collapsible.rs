use crate::button::button_attributes;
use crate::id::ElementId;
use crate::native::after_animations;
use crate::{Controlled, reason, use_id};
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[derive(Clone, Copy)]
struct CollapsibleContext {
    open: Controlled<bool>,
    disabled: Signal<bool>,
    panel: NodeRef<html::Div>,
    panel_id: ElementId<html::Div>,
    // base ui's transition status: idle, starting or ending
    phase: RwSignal<&'static str>,
    shown: RwSignal<bool>,
    size: RwSignal<Option<(i32, i32)>>,
    found: StoredValue<bool>,
}

#[component(element = "div")]
pub fn CollapsibleRoot(
    #[prop(optional_no_strip, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional_no_strip, into)] on_open_change: Option<Callback<(bool, &'static str)>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let state = Controlled::new(open, default_open, on_open_change);
    let panel = NodeRef::new();
    let context = CollapsibleContext {
        open: state,
        disabled,
        panel,
        panel_id: ElementId::new(panel, use_id()),
        phase: RwSignal::new("idle"),
        shown: RwSignal::new(state.get_untracked()),
        size: RwSignal::new(None),
        found: StoredValue::new(false),
    };

    Effect::new(move |previous: Option<bool>| {
        let open = state.get();
        if previous.is_some_and(|previous| previous != open) {
            animate(context, open);
        }
        open
    });
    
    view! {
        <div {..attrs} {..state_attributes(context)} data-disabled=move || disabled.get().then_some("")>
            <Provider value=context>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "button", skip(r#type))]
pub fn CollapsibleTrigger(
    #[prop(optional_no_strip, into)] disabled: Option<Signal<bool>>,
    #[prop(optional_no_strip, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<CollapsibleContext>();
    let disabled =
        move || disabled.map_or_else(|| context.disabled.get(), |disabled| disabled.get());
    let open = move || context.open.get();
    view! {
        <button
            {..attrs}
            {..button_attributes(disabled, || true, tabindex)}
            {..phase_attributes(context)}
            type="button"
            aria-controls=move || open().then(|| context.panel_id.get())
            aria-expanded=move || open().to_string()
            data-panel-open=move || open().then_some("")
            on:click=move |_| context.open.set(!context.open.get_untracked(), reason::TRIGGER_PRESS)
        >
            {children.map(|children| children())}
        </button>
    }
}

#[component(element = "div", skip(id))]
pub fn CollapsiblePanel(
    #[prop(optional_no_strip, into)] id: Option<String>,
    #[prop(optional, into)] hidden_until_found: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<CollapsibleContext>();
    // find in page shows it at once, so it opens without the animation
    let found =
        leptos::tachys::html::event::on(ev::Custom::<ev::Event>::new("beforematch"), move |_| {
            context.found.set_value(true);
            context.open.set(true, reason::NONE);
            if !context.open.get_untracked() {
                context.found.set_value(false);
            }
        });
    let dimension = move |pick: fn((i32, i32)) -> i32| {
        move || {
            context
                .size
                .get()
                .map_or_else(|| "auto".to_string(), |size| format!("{}px", pick(size)))
        }
    };
    view! {
        <div
            {..attrs}
            {..state_attributes(context)}
            {..found}
            node_ref=context.panel
            id=id.unwrap_or_else(|| context.panel_id.get())
            hidden=move || {
                (!context.shown.get()).then(|| if hidden_until_found.get() { "until-found" } else { "" })
            }
            data-disabled=move || context.disabled.get().then_some("")
            style:--collapsible-panel-height=dimension(|(height, _)| height)
            style:--collapsible-panel-width=dimension(|(_, width)| width)
        >
            {children.map(|children| children())}
        </div>
    }
}

fn animate(context: CollapsibleContext, open: bool) {
    let CollapsibleContext {
        panel,
        phase,
        shown,
        size,
        found,
        ..
    } = context;
    let Some(Some(element)) = panel.try_get_untracked() else {
        shown.try_set(open);
        return;
    };
    let still = move |open: bool| context.open.try_get_untracked() == Some(open);
    if open {
        if found.try_update_value(std::mem::take).unwrap_or(false) {
            shown.set(true);
            return;
        }

        let _ = element.remove_attribute("hidden");
        size.set(Some(measure(&element)));
        shown.set(true);
        phase.set("starting");
        after_paint(move || {
            if !still(true) {
                return;
            }
            phase.try_set("idle");
            after_animations(element.into(), move || {
                if still(true) {
                    size.try_set(None);
                }
            });
        });
    } else {
        // auto can't animate to the ending style, so it's pinned to its size first
        size.set(Some(measure(&element)));
        after_paint(move || {
            if !still(false) {
                return;
            }
            phase.try_set("ending");
            after_animations(element.into(), move || {
                if still(false) {
                    shown.try_set(false);
                    phase.try_set("idle");
                    size.try_set(None);
                }
            });
        });
    }
}

// firefox styles the next frame without the one before, so a transition needs a frame painted in between
fn after_paint(done: impl FnOnce() + 'static) {
    request_animation_frame(move || request_animation_frame(done));
}

fn measure(element: &web_sys::HtmlDivElement) -> (i32, i32) {
    (element.scroll_height(), element.scroll_width())
}

fn phase_attributes(context: CollapsibleContext) -> impl leptos::attr::Attribute {
    let phase = context.phase;
    view! {
        <{..}
            data-starting-style=move || (phase.get() == "starting").then_some("")
            data-ending-style=move || (phase.get() == "ending").then_some("")
        />
    }
}

fn state_attributes(context: CollapsibleContext) -> impl leptos::attr::Attribute {
    let open = context.open;
    let attributes = view! {
        <{..}
            data-open=move || open.get().then_some("")
            data-closed=move || (!open.get()).then_some("")
        />
    };
    (attributes, phase_attributes(context))
}
