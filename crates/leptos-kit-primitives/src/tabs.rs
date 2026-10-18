use crate::button::button_attributes;
use crate::composite::Composite;
use crate::id::ElementId;
use crate::native::{after_animations, after_paint};
use crate::{Controlled, reason, use_id};
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[derive(Clone, Copy)]
struct Tab {
    key: usize,
    value: StoredValue<String>,
    disabled: Signal<bool>,
    element: NodeRef<html::Button>,
    id: ElementId<html::Button>,
}

#[derive(Clone, Copy)]
struct Panel {
    key: usize,
    value: StoredValue<String>,
    id: ElementId<html::Div>,
}

#[derive(Clone, Copy)]
struct TabsContext {
    value: Controlled<Option<String>>,
    selected: Memo<Option<String>>,
    // the selection and which way it last moved
    moved: Memo<(Option<String>, &'static str)>,
    orientation: Signal<&'static str>,
    tabs: RwSignal<Vec<Tab>>,
    panels: RwSignal<Vec<Panel>>,
    next: StoredValue<usize>,
}

impl TabsContext {
    fn key(&self) -> usize {
        self.next
            .try_update_value(|next| {
                *next += 1;
                *next - 1
            })
            .unwrap_or_default()
    }

    fn is_selected(&self, value: StoredValue<String>) -> bool {
        self.selected
            .with(|selected| value.with_value(|value| selected.as_ref() == Some(value)))
    }

    fn direction(self) -> impl Fn() -> &'static str + Copy + Send + Sync + 'static {
        move || self.moved.get().1
    }
}

#[derive(Clone, Copy)]
struct ListContext {
    composite: Composite,
    activate_on_focus: Signal<bool>,
}

#[component(element = "div")]
pub fn TabsRoot(
    #[prop(optional_no_strip, into)] value: Option<Signal<Option<String>>>,
    #[prop(optional_no_strip, into)] default_value: Option<String>,
    #[prop(optional_no_strip, into)] on_value_change: Option<Callback<(String, &'static str)>>,
    #[prop(into, default = "horizontal".into())] orientation: Signal<&'static str>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let on_change = on_value_change.map(|on_value_change| {
        Callback::new(move |(value, reason): (Option<String>, &'static str)| {
            if let Some(value) = value {
                on_value_change.run((value, reason));
            }
        })
    });
    let kept = StoredValue::new(default_value.clone());
    let value = Controlled::new(value, default_value, on_change);
    let tabs = RwSignal::new(Vec::<Tab>::new());
    // uncontrolled, a missing or disabled tab gives way to the first enabled one, but a disabled default stays
    let selected = Memo::new(move |_| {
        let current = value.get();
        if value.is_controlled() {
            return current;
        }
        tabs.with(|tabs| {
            let first = || {
                tabs.iter()
                    .find(|tab| !tab.disabled.get())
                    .map(|tab| tab.value.get_value())
            };
            let Some(current) = current else {
                return first();
            };
            let tab = tabs
                .iter()
                .find(|tab| tab.value.with_value(|value| *value == current));
            match tab {
                Some(tab)
                    if tab.disabled.get()
                        && kept.with_value(|kept| kept.as_ref() != Some(&current)) =>
                {
                    first()
                }
                None if !tabs.is_empty() => first(),
                _ => Some(current),
            }
        })
    });
    let moved = Memo::new(move |previous: Option<&(Option<String>, &'static str)>| {
        let now = selected.get();
        match previous {
            Some((before, direction)) if *before == now => (now, *direction),
            Some((before, _)) => {
                let direction = tabs.with_untracked(|tabs| {
                    direction(tabs, before, &now, orientation.get_untracked())
                });
                (now, direction)
            }
            None => (now, "none"),
        }
    });
    let context = TabsContext {
        value,
        selected,
        moved,
        orientation,
        tabs,
        panels: RwSignal::new(Vec::new()),
        next: StoredValue::new(0),
    };
    view! {
        <div
            {..attrs}
            data-orientation=orientation
            data-activation-direction=context.direction()
        >
            <Provider value=context>{children.map(|children| children())}</Provider>
        </div>
    }
}

// base ui compares where the old and the new tab sit
fn direction(
    tabs: &[Tab],
    before: &Option<String>,
    now: &Option<String>,
    orientation: &str,
) -> &'static str {
    let rect = |value: &Option<String>| {
        let value = value.as_ref()?;
        tabs.iter()
            .find(|tab| tab.value.with_value(|tab| tab == value))?
            .element
            .get_untracked()
            .map(|element| element.get_bounding_client_rect())
    };
    let (Some(before), Some(now)) = (rect(before), rect(now)) else {
        return "none";
    };
    let (before, now, backward, forward) = if orientation == "horizontal" {
        (before.left(), now.left(), "left", "right")
    } else {
        (before.top(), now.top(), "up", "down")
    };
    if now < before {
        backward
    } else if now > before {
        forward
    } else {
        "none"
    }
}

#[component(element = "div", skip(role))]
pub fn TabsList(
    #[prop(optional, into)] activate_on_focus: Signal<bool>,
    #[prop(into, default = true.into())] loop_focus: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let tabs = expect_context::<TabsContext>();
    // base ui's arrows reach disabled tabs too
    let composite = Composite::new(tabs.orientation, loop_focus)
        .home_end()
        .focus_disabled();
    let list = ListContext {
        composite,
        activate_on_focus,
    };
    let orientation = tabs.orientation;
    view! {
        <div
            {..attrs}
            {..composite.root_attributes()}
            role="tablist"
            aria-orientation=move || (orientation.get() == "vertical").then_some("vertical")
            data-orientation=orientation
            data-activation-direction=tabs.direction()
        >
            <Provider value=list>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "button", skip(r#type, role, id, tabindex))]
pub fn TabsTab(
    #[prop(into)] value: String,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional_no_strip, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let tabs = expect_context::<TabsContext>();
    let list = expect_context::<ListContext>();
    let value = StoredValue::new(value);
    let element = NodeRef::new();
    let tab = Tab {
        key: tabs.key(),
        value,
        disabled,
        element,
        id: ElementId::new(element, id.unwrap_or_else(use_id)),
    };
    tabs.tabs.update(|all| all.push(tab));
    on_cleanup(move || {
        tabs.tabs
            .try_update(|all| all.retain(|other| other.key != tab.key));
    });
    let active = Signal::derive(move || tabs.is_selected(value));
    let item = list.composite.item(disabled, active, element);
    let idle = move || !active.get_untracked() && !disabled.get_untracked();
    let activate = move || tabs.value.set(Some(value.get_value()), reason::NONE);
    // a focus that comes with a press other than the main button doesn't activate
    let pressing = StoredValue::new(None::<bool>);
    let panel_id = move || {
        tabs.panels.with(|panels| {
            panels
                .iter()
                .find(|panel| {
                    panel
                        .value
                        .with_value(|panel| value.with_value(|value| panel == value))
                })
                .map(|panel| panel.id.get())
        })
    };
    view! {
        <button
            {..attrs}
            {..button_attributes(
                move || disabled.get(),
                || true,
                Some(list.composite.tabindex(item)),
            )}
            {..list.composite.item_attributes(item)}
            node_ref=element
            type="button"
            role="tab"
            id=tab.id.get()
            aria-selected=move || active.get().to_string()
            aria-controls=move || active.get().then(panel_id).flatten()
            data-active=move || active.get().then_some("")
            data-orientation=tabs.orientation
            data-activation-direction=tabs.direction()
            on:click=move |_| {
                if idle() {
                    activate();
                }
            }
            on:focus=move |_| {
                let pressed = pressing.try_update_value(Option::take).flatten();
                if idle() && list.activate_on_focus.get_untracked() && pressed != Some(false) {
                    activate();
                }
            }
            on:pointerdown=move |event: ev::PointerEvent| {
                if idle() {
                    pressing.set_value(Some(event.button() == 0));
                }
            }
            on:pointerup=move |_| pressing.set_value(None)
            on:pointercancel=move |_| pressing.set_value(None)
        >
            {children.map(|children| children())}
        </button>
    }
}

// always in the dom, hidden when not selected, so there's no keep_mounted
#[component(element = "div", skip(id, role, tabindex))]
pub fn TabsPanel(
    #[prop(into)] value: String,
    #[prop(optional_no_strip, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let tabs = expect_context::<TabsContext>();
    let value = StoredValue::new(value);
    let element = NodeRef::new();
    let panel = Panel {
        key: tabs.key(),
        value,
        id: ElementId::new(element, id.unwrap_or_else(use_id)),
    };
    let index = tabs.panels.with_untracked(Vec::len);
    tabs.panels.update(|all| all.push(panel));
    on_cleanup(move || {
        tabs.panels
            .try_update(|all| all.retain(|other| other.key != panel.key));
    });
    let open = Signal::derive(move || tabs.is_selected(value));
    let (shown, phase) = presence(open, element);
    let tab_id = move || {
        tabs.tabs.with(|all| {
            all.iter()
                .find(|tab| {
                    tab.value
                        .with_value(|tab| value.with_value(|value| tab == value))
                })
                .map(|tab| tab.id.get())
        })
    };
    view! {
        <div
            {..attrs}
            node_ref=element
            id=panel.id.get()
            role="tabpanel"
            aria-labelledby=tab_id
            tabindex=move || if open.get() { 0 } else { -1 }
            hidden=move || !shown.get()
            inert=move || !open.get()
            data-hidden=move || (!shown.get()).then_some("")
            data-index=index
            data-orientation=tabs.orientation
            data-activation-direction=tabs.direction()
            data-starting-style=move || (phase.get() == "starting").then_some("")
            data-ending-style=move || (phase.get() == "ending").then_some("")
        >
            {children.map(|children| children())}
        </div>
    }
}

// the old panel plays its ending style while the new one starts, as base ui's transition status
fn presence(
    open: Signal<bool>,
    element: NodeRef<html::Div>,
) -> (RwSignal<bool>, RwSignal<&'static str>) {
    let shown = RwSignal::new(open.get_untracked());
    let phase = RwSignal::new("idle");
    Effect::new(move |previous: Option<bool>| {
        let now = open.get();
        if previous.is_none_or(|previous| previous == now) {
            return now;
        }
        let still = move || open.try_get_untracked() == Some(now);
        if now {
            shown.set(true);
            phase.set("starting");
            after_paint(move || {
                if still() {
                    phase.try_set("idle");
                }
            });
        } else if let Some(element) = element.get_untracked() {
            phase.set("ending");
            after_animations(element.into(), move || {
                if still() {
                    shown.try_set(false);
                    phase.try_set("idle");
                }
            });
        } else {
            shown.set(false);
        }
        now
    });
    (shown, phase)
}
