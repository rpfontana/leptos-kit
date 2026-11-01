use crate::button::button_attributes;
use crate::dismiss::dismiss;
use crate::hover::Hover;
use crate::id::ElementId;
use crate::native::{after_animations, after_paint, observe_resize};
use crate::transition::{TransitionStatus, transition_attributes};
use crate::{
    Align, ChangeReason, Controlled, Direction, Keyword, Orientation, Side, use_direction, use_id,
};
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_kit_macros::component;
use web_sys::{Element, HtmlElement, Node};

// base ui: a click this soon after a hover opened the popup leaves it open
const PATIENT_CLICK_MS: f64 = 500.0;
const COLLISION_PADDING: f64 = 5.0;
const GUARD_STYLE: &str = "clip-path: inset(50%); overflow: hidden; white-space: nowrap; border: 0px; \
     padding: 0px; width: 1px; height: 1px; margin: -1px; position: fixed; top: 0px; left: 0px;";
const OWNER_STYLE: &str = "clip-path: inset(50%); position: fixed; top: 0px; left: 0px;";
const POSITIONER_STYLE: &str = "position:absolute;inset:auto;margin:0;padding:0;border:0;\
     background:transparent;color:inherit;overflow:visible";
const TABBABLE: &str = "a[href], button, input, select, textarea, [tabindex], [contenteditable]";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Keyword)]
enum Activation {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Copy)]
struct TriggerEntry {
    value: StoredValue<String>,
    element: NodeRef<html::Button>,
    before: NodeRef<html::Span>,
    after: NodeRef<html::Span>,
}

#[derive(Clone, Copy)]
struct Placing {
    side: Signal<Side>,
    align: Signal<Align>,
    side_offset: Signal<f64>,
    align_offset: Signal<f64>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct NavigationMenuContext(StoredValue<NavigationMenu>);

#[derive(Clone, Copy)]
struct NavigationMenu {
    value: Controlled<Option<String>>,
    orientation: Signal<Orientation>,
    direction: Signal<Direction>,
    delay: Signal<u32>,
    close_delay: Signal<u32>,
    on_open_change_complete: Option<Callback<bool>>,
    mounted: RwSignal<bool>,
    phase: RwSignal<TransitionStatus>,
    activation: RwSignal<Option<Activation>>,
    instant: RwSignal<bool>,
    shown_side: RwSignal<Side>,
    shown_align: RwSignal<Align>,
    placing: RwSignal<Option<Placing>>,
    root: NodeRef<html::Nav>,
    positioner: NodeRef<html::Div>,
    popup: NodeRef<html::Nav>,
    viewport: NodeRef<html::Div>,
    before_inside: NodeRef<html::Span>,
    after_inside: NodeRef<html::Span>,
    popup_id: ElementId<html::Nav>,
    viewport_id: ElementId<html::Div>,
    triggers: StoredValue<Vec<TriggerEntry>>,
    previous: StoredValue<Option<String>>,
    close_reason: StoredValue<Option<ChangeReason>>,
    hover: Hover,
    hovered_at: StoredValue<Option<f64>>,
    touch: StoredValue<bool>,
    sizing: StoredValue<u64>,
}

impl NavigationMenuContext {
    fn menu(self) -> NavigationMenu {
        self.0.get_value()
    }

    fn try_menu(self) -> Option<NavigationMenu> {
        self.0.try_get_value()
    }

    fn open(self) -> bool {
        self.try_menu()
            .is_some_and(|menu| menu.value.with(Option::is_some))
    }

    fn active(self, item: StoredValue<String>) -> bool {
        self.try_menu().is_some_and(|menu| {
            menu.value
                .with(|value| item.with_value(|item| value.as_deref() == Some(item.as_str())))
        })
    }

    fn open_now(self) -> bool {
        untrack(|| self.open())
    }

    fn active_now(self, item: StoredValue<String>) -> bool {
        untrack(|| self.active(item))
    }

    fn entry(self, value: &str) -> Option<TriggerEntry> {
        self.try_menu()?.triggers.with_value(|triggers| {
            triggers
                .iter()
                .find(|entry| entry.value.with_value(|own| own == value))
                .copied()
        })
    }

    fn trigger_of(self, value: &str) -> Option<HtmlElement> {
        self.entry(value)?.element.get_untracked().map(Into::into)
    }

    fn active_trigger(self) -> Option<HtmlElement> {
        let value = self.try_menu()?.value.get_untracked()?;
        self.trigger_of(&value)
    }

    fn popup(self) -> Option<HtmlElement> {
        self.try_menu()?.popup.get_untracked()
    }

    fn positioner(self) -> Option<HtmlElement> {
        self.try_menu()?.positioner.get_untracked().map(Into::into)
    }

    fn inside(self, node: Option<&Node>) -> bool {
        let Some(node) = node else {
            return false;
        };
        let in_root = self
            .try_menu()
            .and_then(|menu| menu.root.get_untracked())
            .is_some_and(|root| root.contains(Some(node)));
        in_root
            || self
                .positioner()
                .is_some_and(|positioner| positioner.contains(Some(node)))
    }

    fn activate(self, item: &str, reason: ChangeReason) {
        let Some(menu) = self.try_menu() else {
            return;
        };
        // a close the pointer scheduled doesn't outlive an item opened since
        menu.hover.cancel();
        let current = menu.value.get_untracked();
        if current.as_deref() == Some(item) {
            return;
        }
        let from = current
            .as_ref()
            .and_then(|_| self.popup())
            .map(|popup| size(&popup));
        let activation = current
            .as_deref()
            .and_then(|current| self.trigger_of(current))
            .zip(self.trigger_of(item))
            .and_then(|(was, next)| {
                let (was, next) = (
                    was.get_bounding_client_rect(),
                    next.get_bounding_client_rect(),
                );
                match menu.orientation.get_untracked() {
                    Orientation::Horizontal if next.left() != was.left() => {
                        Some(if next.left() > was.left() {
                            Activation::Right
                        } else {
                            Activation::Left
                        })
                    }
                    Orientation::Vertical if next.top() != was.top() => {
                        Some(if next.top() > was.top() {
                            Activation::Down
                        } else {
                            Activation::Up
                        })
                    }
                    _ => None,
                }
            });
        if let Some(activation) = activation {
            menu.activation.set(Some(activation));
        }
        menu.value.set(Some(item.to_string()), reason);
        if menu
            .value
            .with_untracked(|value| value.as_deref() == Some(item))
        {
            menu.previous.set_value(Some(item.to_string()));
            self.resize(from);
        }
    }

    fn close(self, reason: ChangeReason) {
        let Some(menu) = self.try_menu() else {
            return;
        };
        if menu.value.with_untracked(Option::is_none) {
            return;
        }
        menu.close_reason.set_value(Some(reason));
        menu.hover.cancel();
        menu.value.set(None, reason);
        if menu.value.with_untracked(Option::is_some) {
            return;
        }
        menu.activation.set(None);
        // the popup keeps the positioner's size while its content leaves
        if let (Some(popup), Some(positioner)) = (self.popup(), self.positioner()) {
            let style = positioner.style();
            let width = style
                .get_property_value("--positioner-width")
                .unwrap_or_default();
            let height = style
                .get_property_value("--positioner-height")
                .unwrap_or_default();
            if !width.is_empty() && !height.is_empty() {
                let _ = popup.style().set_property("--popup-width", &width);
                let _ = popup.style().set_property("--popup-height", &height);
            }
        }
    }

    // base ui sizes the popup to its new content and animates there from the size it had
    fn resize(self, from: Option<(f64, f64)>) {
        let Some(menu) = self.try_menu() else {
            return;
        };
        let generation = menu.sizing.get_value() + 1;
        menu.sizing.set_value(generation);
        let current = move || {
            self.try_menu()
                .is_some_and(|menu| menu.sizing.get_value() == generation)
        };
        request_animation_frame(move || {
            let (Some(popup), Some(positioner)) = (self.popup(), self.positioner()) else {
                return;
            };
            if !current() {
                return;
            }
            set_sizes(&popup, "--popup", None);
            set_sizes(&positioner, "--positioner", None);
            let (width, height) = size(&popup);
            set_sizes(&positioner, "--positioner", Some((width, height)));
            self.place(width, height, from.is_none());
            match from {
                Some((was_width, was_height)) if was_width > 0.0 && was_height > 0.0 => {
                    set_sizes(&popup, "--popup", Some((was_width, was_height)));
                    request_animation_frame(move || {
                        if !current() {
                            return;
                        }
                        set_sizes(&popup, "--popup", Some((width, height)));
                        let target = popup.clone();
                        after_animations(target, move || {
                            if current() {
                                set_auto(&popup);
                            }
                        });
                    });
                }
                _ => set_auto(&popup),
            }
        });
    }

    // floating-ui's placement against the active trigger: flip the side or the alignment, then shift onto the screen
    fn place(self, width: f64, height: f64, instant: bool) {
        let (Some(menu), Some(trigger), Some(positioner)) =
            (self.try_menu(), self.active_trigger(), self.positioner())
        else {
            return;
        };
        let Some(placing) = menu.placing.get_untracked() else {
            return;
        };
        let Some(viewport) = document().document_element() else {
            return;
        };
        let (screen_width, screen_height) = (
            f64::from(viewport.client_width()),
            f64::from(viewport.client_height()),
        );
        let rect = trigger.get_bounding_client_rect();
        let rtl = menu.direction.get_untracked() == Direction::Rtl;
        let offset = placing.side_offset.get_untracked();
        let align_offset = placing.align_offset.get_untracked();
        let padding = COLLISION_PADDING;
        let mut side = match placing.side.get_untracked() {
            Side::InlineStart if rtl => Side::Right,
            Side::InlineStart => Side::Left,
            Side::InlineEnd if rtl => Side::Left,
            Side::InlineEnd => Side::Right,
            side => side,
        };
        let room = |side: Side| match side {
            Side::Top => rect.top() - offset - padding,
            Side::Left => rect.left() - offset - padding,
            Side::Right => screen_width - rect.right() - offset - padding,
            _ => screen_height - rect.bottom() - offset - padding,
        };
        let opposite = match side {
            Side::Top => Side::Bottom,
            Side::Left => Side::Right,
            Side::Right => Side::Left,
            _ => Side::Top,
        };
        let needed = if matches!(side, Side::Top | Side::Bottom) {
            height
        } else {
            width
        };
        if room(side) < needed && room(opposite) >= needed {
            side = opposite;
        }
        let vertical = matches!(side, Side::Top | Side::Bottom);
        let (start, length, extent, screen) = if vertical {
            (rect.left(), rect.width(), width, screen_width)
        } else {
            (rect.top(), rect.height(), height, screen_height)
        };
        let at = |align: Align| match align {
            Align::Start => start + align_offset,
            Align::Center => start + (length - extent) / 2.0 + align_offset,
            Align::End => start + length - extent - align_offset,
        };
        let mut align = placing.align.get_untracked();
        let fits = |position: f64| position >= padding && position + extent <= screen - padding;
        let flipped = match align {
            Align::Start => Align::End,
            Align::End => Align::Start,
            Align::Center => Align::Center,
        };
        if !fits(at(align)) && fits(at(flipped)) {
            align = flipped;
        }
        let across = at(align).clamp(padding, (screen - padding - extent).max(padding));
        let along = match side {
            Side::Top => rect.top() - offset - height,
            Side::Left => rect.left() - offset - width,
            Side::Right => rect.right() + offset,
            _ => rect.bottom() + offset,
        };
        let (left, top) = if vertical {
            (across, along)
        } else {
            (along, across)
        };
        let (scroll_x, scroll_y) = (
            window().scroll_x().unwrap_or_default(),
            window().scroll_y().unwrap_or_default(),
        );
        let style = positioner.style();
        if instant {
            let _ = style.set_property("transition", "none");
        }
        let _ = style.set_property("left", &format!("{}px", left + scroll_x));
        let _ = style.set_property("top", &format!("{}px", top + scroll_y));
        let (available_width, available_height) = match side {
            Side::Top => (screen_width - 2.0 * padding, rect.top() - offset - padding),
            Side::Left => (
                rect.left() - offset - padding,
                screen_height - 2.0 * padding,
            ),
            Side::Right => (
                screen_width - rect.right() - offset - padding,
                screen_height - 2.0 * padding,
            ),
            _ => (
                screen_width - 2.0 * padding,
                screen_height - rect.bottom() - offset - padding,
            ),
        };
        let _ = style.set_property("--available-width", &format!("{available_width}px"));
        let _ = style.set_property("--available-height", &format!("{available_height}px"));
        let _ = style.set_property("--anchor-width", &format!("{}px", rect.width()));
        let _ = style.set_property("--anchor-height", &format!("{}px", rect.height()));
        let portion = |align: Align| match align {
            Align::Start => "0%",
            Align::Center => "50%",
            Align::End => "100%",
        };
        let origin = match side {
            Side::Top => format!("{} {}px", portion(align), height + offset),
            Side::Left => format!("{}px {}", width + offset, portion(align)),
            Side::Right => format!("-{offset}px {}", portion(align)),
            _ => format!("{} -{offset}px", portion(align)),
        };
        let _ = style.set_property("--transform-origin", &origin);
        menu.shown_side.set(side);
        menu.shown_align.set(align);
        if instant {
            menu.instant.set(true);
            let _ = window().get_computed_style(&positioner);
            let _ = positioner.offset_width();
            let _ = style.remove_property("transition");
            after_paint(move || {
                if let Some(menu) = self.try_menu() {
                    menu.instant.set(false);
                }
            });
        }
    }

    fn closed(self) {
        let Some(menu) = self.try_menu() else {
            return;
        };
        let reason = menu.close_reason.try_update_value(Option::take).flatten();
        let blocked = matches!(
            reason,
            Some(ChangeReason::TriggerHover | ChangeReason::OutsidePress | ChangeReason::FocusOut)
        );
        let stranded = match document().active_element() {
            Some(active) => {
                active.tag_name().eq_ignore_ascii_case("body")
                    || self
                        .positioner()
                        .is_some_and(|positioner| positioner.contains(Some(&active)))
            }
            None => true,
        };
        if !blocked
            && stranded
            && let Some(trigger) = menu
                .previous
                .get_value()
                .and_then(|previous| self.trigger_of(&previous))
        {
            focus(&trigger);
        }
        menu.phase.set(TransitionStatus::Idle);
        menu.activation.set(None);
        menu.touch.set_value(false);
        if let Some(popup) = self.popup() {
            set_auto(&popup);
        }
        if let Some(on_open_change_complete) = menu.on_open_change_complete {
            on_open_change_complete.run(false);
        }
    }
}

fn size(element: &HtmlElement) -> (f64, f64) {
    let Ok(Some(style)) = window().get_computed_style(element) else {
        return (0.0, 0.0);
    };
    let read = |name: &str| {
        style
            .get_property_value(name)
            .ok()
            .and_then(|value| value.trim_end_matches("px").parse::<f64>().ok())
            .unwrap_or_default()
    };
    (read("width"), read("height"))
}

fn set_sizes(element: &HtmlElement, prefix: &str, size: Option<(f64, f64)>) {
    let style = element.style();
    let (width, height) = (format!("{prefix}-width"), format!("{prefix}-height"));
    if let Some((w, h)) = size {
        let _ = style.set_property(&width, &format!("{w}px"));
        let _ = style.set_property(&height, &format!("{h}px"));
    } else {
        let _ = style.remove_property(&width);
        let _ = style.remove_property(&height);
    }
}

fn set_auto(popup: &HtmlElement) {
    let style = popup.style();
    let _ = style.set_property("--popup-width", "auto");
    let _ = style.set_property("--popup-height", "auto");
}

fn focus(element: &HtmlElement) {
    let options = web_sys::FocusOptions::new();
    options.set_prevent_scroll(true);
    let _ = element.focus_with_options(&options);
}

fn related(event: &ev::FocusEvent) -> Option<Node> {
    event
        .related_target()
        .and_then(|target| target.dyn_into::<Node>().ok())
}

// the tabbable library's order, relative to what has the focus
fn tabbable(step: isize) -> Option<HtmlElement> {
    let list = document().query_selector_all(TABBABLE).ok()?;
    let tabbables: Vec<HtmlElement> = (0..list.length())
        .filter_map(|at| list.item(at)?.dyn_into::<HtmlElement>().ok())
        .filter(|element| {
            let rect = element.get_bounding_client_rect();
            element.tab_index() >= 0
                && !element.matches(":disabled").unwrap_or(false)
                && element
                    .closest("[hidden], [inert]")
                    .ok()
                    .flatten()
                    .is_none()
                && (rect.width() > 0.0 || rect.height() > 0.0)
        })
        .collect();
    let active = document().active_element();
    let index = tabbables.iter().position(|element| {
        active
            .as_ref()
            .is_some_and(|active| element.is_same_node(Some(active)))
    });
    let target = match index {
        Some(index) => isize::try_from(index).ok()? + step,
        None if step > 0 => 0,
        None => isize::try_from(tabbables.len()).ok()? - 1,
    };
    tabbables.get(usize::try_from(target).ok()?).cloned()
}

fn step_focus(container: &Element, items: &str, forward: bool, looping: bool) {
    let Ok(list) = container.query_selector_all(items) else {
        return;
    };
    let items: Vec<HtmlElement> = (0..list.length())
        .filter_map(|at| list.item(at)?.dyn_into::<HtmlElement>().ok())
        .filter(|element| element.closest("[hidden]").ok().flatten().is_none())
        .collect();
    let active = document().active_element();
    let Some(index) = items.iter().position(|element| {
        active
            .as_ref()
            .is_some_and(|active| element.is_same_node(Some(active)))
    }) else {
        return;
    };
    let next = if forward {
        match items.get(index + 1) {
            Some(next) => Some(next),
            None if looping => items.first(),
            None => None,
        }
    } else {
        match index.checked_sub(1) {
            Some(previous) => items.get(previous),
            None if looping => items.last(),
            None => None,
        }
    };
    if let Some(next) = next {
        let _ = next.focus();
    }
}

#[component(element = "nav")]
pub fn NavigationMenuRoot(
    #[prop(optional, into)] value: Option<Signal<Option<String>>>,
    #[prop(optional, into)] default_value: Option<String>,
    #[prop(optional, into)] on_value_change: Option<Callback<(Option<String>, ChangeReason)>>,
    #[prop(into, default = 50.into())] delay: Signal<u32>,
    #[prop(into, default = 50.into())] close_delay: Signal<u32>,
    #[prop(into, default = Orientation::Horizontal.into())] orientation: Signal<Orientation>,
    #[prop(optional, into)] on_open_change_complete: Option<Callback<bool>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let base = use_id();
    let popup = NodeRef::new();
    let viewport = NodeRef::new();
    let context = NavigationMenuContext(StoredValue::new(NavigationMenu {
        value: Controlled::new(value, default_value, on_value_change),
        orientation,
        direction: use_direction(),
        delay,
        close_delay,
        on_open_change_complete,
        mounted: RwSignal::new(false),
        phase: RwSignal::new(TransitionStatus::Idle),
        activation: RwSignal::new(None),
        instant: RwSignal::new(false),
        shown_side: RwSignal::new(Side::Bottom),
        shown_align: RwSignal::new(Align::Center),
        placing: RwSignal::new(None),
        root: NodeRef::new(),
        positioner: NodeRef::new(),
        popup,
        viewport,
        before_inside: NodeRef::new(),
        after_inside: NodeRef::new(),
        popup_id: ElementId::new(popup, format!("{base}-popup")),
        viewport_id: ElementId::new(viewport, format!("{base}-viewport")),
        triggers: StoredValue::new(Vec::new()),
        previous: StoredValue::new(None),
        close_reason: StoredValue::new(None),
        hover: Hover::new(),
        hovered_at: StoredValue::new(None),
        touch: StoredValue::new(false),
        sizing: StoredValue::new(0),
    }));
    let menu = context.menu();
    let open = move || context.open();
    // into the top layer at once, out of it once the popup's exit animation ends
    Effect::new(move |was_open: Option<bool>| {
        let open = context.open();
        let Some(positioner) = menu.positioner.get() else {
            return open;
        };
        let positioner: HtmlElement = positioner.into();
        let shown = positioner.matches(":popover-open").unwrap_or(false);
        if open {
            if !shown {
                let _ = positioner.show_popover();
                menu.mounted.set(true);
                context.resize(None);
            }
        } else if was_open == Some(true) && shown {
            menu.phase.set(TransitionStatus::Ending);
            let generation = menu.sizing.get_value() + 1;
            menu.sizing.set_value(generation);
            let animated = context.popup().unwrap_or_else(|| positioner.clone());
            after_animations(animated, move || {
                let Some(menu) = context.try_menu() else {
                    return;
                };
                if menu.sizing.get_value() == generation && !context.open_now() {
                    let _ = positioner.hide_popover();
                }
            });
        }
        open
    });
    dismiss(
        open,
        move |node| {
            // a press on any navigation menu's trigger leaves it to that trigger
            context.inside(Some(node))
                || node
                    .dyn_ref::<Element>()
                    .and_then(|element| {
                        element
                            .closest("[data-base-ui-navigation-menu-trigger]")
                            .ok()
                            .flatten()
                    })
                    .is_some()
        },
        move |reason| context.close(reason),
    );
    let resize = window_event_listener(ev::resize, move |_| {
        if context.open_now()
            && let Some(popup) = context.popup()
        {
            let (width, height) = size(&popup);
            if let Some(positioner) = context.positioner() {
                set_sizes(&positioner, "--positioner", Some((width, height)));
            }
            context.place(width, height, true);
        }
    });
    on_cleanup(move || resize.remove());
    view! {
        <nav {..attrs} node_ref=menu.root data-open=move || open().then_some("")>
            <Provider value=context>{children.map(|children| children())}</Provider>
        </nav>
    }
}

#[component(element = "ul")]
pub fn NavigationMenuList(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<NavigationMenuContext>();
    let menu = context.menu();
    view! {
        <ul
            {..attrs}
            data-open=move || context.open().then_some("")
            on:keydown=move |event: ev::KeyboardEvent| {
                let rtl = menu.direction.get_untracked() == Direction::Rtl;
                let (back, forward) = match menu.orientation.get_untracked() {
                    Orientation::Horizontal if rtl => ("ArrowRight", "ArrowLeft"),
                    Orientation::Horizontal => ("ArrowLeft", "ArrowRight"),
                    Orientation::Vertical => ("ArrowUp", "ArrowDown"),
                };
                let key = event.key();
                if key != back && key != forward {
                    return;
                }
                event.stop_propagation();
                event.prevent_default();
                if let Some(list) = event.current_target().and_then(|target| target.dyn_into::<Element>().ok()) {
                    step_focus(&list, "[data-base-ui-navigation-menu-trigger], [data-navigation-menu-link]", key == forward, false);
                }
            }
        >
            {children.map(|children| children())}
        </ul>
    }
}

#[derive(Clone, Copy)]
struct ItemContext {
    value: StoredValue<String>,
    before: NodeRef<html::Span>,
    after: NodeRef<html::Span>,
}

#[component(element = "li")]
pub fn NavigationMenuItem(
    #[prop(optional, into)] value: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<NavigationMenuContext>();
    let menu = context.menu();
    let item = ItemContext {
        value: StoredValue::new(value.unwrap_or_else(use_id)),
        before: NodeRef::new(),
        after: NodeRef::new(),
    };
    let active = move || context.active(item.value);
    let from_popup = move |event: &ev::FocusEvent| {
        context
            .positioner()
            .is_some_and(|positioner| positioner.contains(related(event).as_ref()))
    };
    view! {
        <li {..attrs}>
            <Provider value=item>{children.map(|children| children())}</Provider>
            // base ui's focus guards: tab from the active trigger goes through its popup, which sits elsewhere
            <span
                node_ref=item.before
                aria-hidden="true"
                tabindex="0"
                data-base-ui-focus-guard=""
                style=GUARD_STYLE
                hidden=move || !active()
                on:focus=move |event: ev::FocusEvent| {
                    if from_popup(&event) {
                        if let Some(trigger) = context.trigger_of(&item.value.get_value()) {
                            let _ = trigger.focus();
                        }
                    } else if let Some(guard) = menu.before_inside.get_untracked() {
                        let _ = guard.focus();
                    }
                }
            ></span>
            <span
                aria-owns=move || active().then(|| menu.viewport_id.get())
                style=OWNER_STYLE
                hidden=move || !active()
            ></span>
            <span
                node_ref=item.after
                aria-hidden="true"
                tabindex="0"
                data-base-ui-focus-guard=""
                style=GUARD_STYLE
                hidden=move || !active()
                on:focus=move |event: ev::FocusEvent| {
                    if !from_popup(&event) {
                        if let Some(guard) = menu.after_inside.get_untracked() {
                            let _ = guard.focus();
                        }
                        return;
                    }
                    let next = tabbable(1);
                    if let Some(next) = &next {
                        let _ = next.focus();
                    }
                    if !context.inside(next.as_ref().map(AsRef::as_ref)) {
                        context.close(ChangeReason::FocusOut);
                    }
                }
            ></span>
        </li>
    }
}

#[component(element = "button", skip(r#type))]
pub fn NavigationMenuTrigger(
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<NavigationMenuContext>();
    let menu = context.menu();
    let ItemContext {
        value: item,
        before,
        after,
    } = expect_context::<ItemContext>();
    let entry = TriggerEntry {
        value: item,
        element: NodeRef::new(),
        before,
        after,
    };
    menu.triggers.update_value(|triggers| triggers.push(entry));
    on_cleanup(move || {
        if let Some(menu) = context.try_menu() {
            menu.triggers
                .try_update_value(|triggers| triggers.retain(|own| own.value != entry.value));
        }
    });
    let active = move || context.active(item);
    let value = move || item.get_value();
    let open_now = move |reason: ChangeReason| {
        if !disabled.get_untracked() {
            context.activate(&value(), reason);
        }
    };
    let leave = move || {
        if let Some(menu) = context.try_menu() {
            menu.hover.leave(
                menu.close_delay.get_untracked(),
                Callback::new(move |()| context.close(ChangeReason::TriggerHover)),
            );
        }
    };
    view! {
        <button
            {..attrs}
            {..button_attributes(move || disabled.get(), || true, None)}
            node_ref=entry.element
            type="button"
            aria-expanded=move || active().to_string()
            aria-controls=move || active().then(|| menu.popup_id.get())
            data-base-ui-navigation-menu-trigger=""
            data-popup-open=move || active().then_some("")
            data-pressed=move || active().then_some("")
            on:pointerenter=move |event: ev::PointerEvent| menu.touch.set_value(event.pointer_type() == "touch")
            on:pointerdown=move |event: ev::PointerEvent| menu.touch.set_value(event.pointer_type() == "touch")
            on:mouseenter=move |_| {
                if disabled.get_untracked() || menu.touch.get_value() {
                    return;
                }
                if context.open_now() {
                    menu.hover.hold();
                    if !context.active_now(item) {
                        open_now(ChangeReason::TriggerHover);
                    }
                    return;
                }
                let delay = menu.delay.get_untracked();
                menu.hover.enter(delay, Callback::new(move |()| {
                    menu.hovered_at.set_value(Some(js_sys_now()));
                    open_now(ChangeReason::TriggerHover);
                }));
            }
            on:mouseleave=move |event: ev::MouseEvent| {
                if menu.touch.get_value() {
                    return;
                }
                let (Some(trigger), Some(popup)) = (entry.element.get_untracked(), context.popup()) else {
                    leave();
                    return;
                };
                if !context.active_now(item) {
                    leave();
                    return;
                }
                let from = (f64::from(event.client_x()), f64::from(event.client_y()));
                menu.hover.cross(
                    from,
                    [trigger.into(), popup.into()],
                    menu.close_delay.get_untracked(),
                    Callback::new(move |()| context.close(ChangeReason::TriggerHover)),
                );
            }
            on:click=move |_| {
                if disabled.get_untracked() {
                    return;
                }
                if context.active_now(item) {
                    // a click right after a hover opened it is the same intent, it stays open
                    let patient = menu
                        .hovered_at
                        .get_value()
                        .is_none_or(|at| js_sys_now() - at >= PATIENT_CLICK_MS);
                    if patient {
                        context.close(ChangeReason::TriggerPress);
                    }
                    return;
                }
                menu.hovered_at.set_value(None);
                open_now(ChangeReason::TriggerPress);
            }
            on:keydown=move |event: ev::KeyboardEvent| {
                let rtl = menu.direction.get_untracked() == Direction::Rtl;
                let opens = match menu.orientation.get_untracked() {
                    Orientation::Horizontal => event.key() == "ArrowDown",
                    Orientation::Vertical => event.key() == if rtl { "ArrowLeft" } else { "ArrowRight" },
                };
                if opens {
                    event.prevent_default();
                    event.stop_propagation();
                    menu.hovered_at.set_value(None);
                    open_now(ChangeReason::ListNavigation);
                }
            }
            on:blur=move |event: ev::FocusEvent| {
                if context.open_now() && !context.inside(related(&event).as_ref()) {
                    context.close(ChangeReason::FocusOut);
                }
            }
        >
            {children.map(|children| children())}
        </button>
    }
}

#[component(element = "span")]
pub fn NavigationMenuIcon(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<NavigationMenuContext>();
    let item = expect_context::<ItemContext>().value;
    let glyph = match children {
        Some(children) => leptos::either::Either::Left(children()),
        None => leptos::either::Either::Right("▼"),
    };
    view! {
        <span {..attrs} aria-hidden="true" data-popup-open=move || context.active(item).then_some("")>
            {glyph}
        </span>
    }
}

#[component(element = "div")]
pub fn NavigationMenuContent(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<NavigationMenuContext>();
    let menu = context.menu();
    let item = expect_context::<ItemContext>().value;
    let element = NodeRef::<html::Div>::new();
    let mounted = RwSignal::new(false);
    let status = RwSignal::new(TransitionStatus::Idle);
    let focus_inside = RwSignal::new(false);
    let open = move || menu.mounted.get() && context.active(item);
    // base ui portals the active content into the viewport; this one moves there once and stays
    Effect::new(move || {
        if let (Some(content), Some(viewport)) = (element.get(), menu.viewport.get())
            && !viewport.contains(Some(&content))
        {
            let _ = viewport.append_child(&content);
        }
    });
    Effect::new(move |was: Option<bool>| {
        let now = open();
        if now {
            mounted.set(true);
            if was != Some(true) {
                status.set(TransitionStatus::Starting);
                after_paint(move || {
                    if status.try_get_untracked() == Some(TransitionStatus::Starting) {
                        status.set(TransitionStatus::Idle);
                    }
                });
            }
        } else if was == Some(true) {
            status.set(TransitionStatus::Ending);
            if let Some(content) = element.get_untracked() {
                after_animations(content.into(), move || {
                    if !untrack(open) {
                        mounted.try_set(false);
                        status.try_set(TransitionStatus::Idle);
                    }
                });
            }
        }
        now
    });
    let leaving = move || !open() && mounted.get();
    view! {
        <div
            {..attrs}
            {..transition_attributes(status)}
            node_ref=element
            hidden=move || !(mounted.get() && menu.mounted.get())
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-activation-direction=move || menu.activation.get().map(Activation::as_str)
            inert=move || leaving() && !focus_inside.get()
            style:position=move || leaving().then_some("absolute")
            style:top=move || leaving().then_some("0")
            style:left=move || leaving().then_some("0")
            on:focusin=move |event: ev::FocusEvent| {
                let guard = event
                    .target()
                    .and_then(|target| target.dyn_into::<Element>().ok())
                    .is_some_and(|target| target.has_attribute("data-base-ui-focus-guard"));
                if !guard {
                    focus_inside.set(true);
                }
            }
            on:focusout=move |event: ev::FocusEvent| {
                let stays = element
                    .get_untracked()
                    .is_some_and(|content| content.contains(related(&event).as_ref()));
                if !stays {
                    focus_inside.set(false);
                }
            }
            on:keydown=move |event: ev::KeyboardEvent| {
                let forward = matches!(event.key().as_str(), "ArrowDown" | "ArrowRight");
                let back = matches!(event.key().as_str(), "ArrowUp" | "ArrowLeft");
                if !forward && !back {
                    return;
                }
                if let Some(content) = element.get_untracked() {
                    event.prevent_default();
                    step_focus(&content, "[data-navigation-menu-link]", forward, true);
                }
            }
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "a")]
pub fn NavigationMenuLink(
    #[prop(optional, into)] active: Signal<bool>,
    #[prop(optional)] close_on_click: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<NavigationMenuContext>();
    view! {
        <a
            {..attrs}
            data-navigation-menu-link=""
            aria-current=move || active.get().then_some("page")
            data-active=move || active.get().then_some("")
            on:click=move |_| {
                if close_on_click {
                    context.close(ChangeReason::LinkPress);
                }
            }
            on:blur=move |event: ev::FocusEvent| {
                if context.open_now() && !context.inside(related(&event).as_ref()) {
                    context.close(ChangeReason::FocusOut);
                }
            }
        >
            {children.map(|children| children())}
        </a>
    }
}

#[leptos::component]
pub fn NavigationMenuPortal(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    children.map(|children| children())
}

#[component(element = "div", skip(role, style))]
pub fn NavigationMenuPositioner(
    #[prop(into, default = Side::Bottom.into())] side: Signal<Side>,
    #[prop(into, default = Align::Center.into())] align: Signal<Align>,
    #[prop(optional, into)] side_offset: Signal<f64>,
    #[prop(optional, into)] align_offset: Signal<f64>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<NavigationMenuContext>();
    let menu = context.menu();
    menu.placing.set(Some(Placing {
        side,
        align,
        side_offset,
        align_offset,
    }));
    let open = move || context.open();
    Effect::new(move || {
        let Some(popup) = menu.popup.get() else {
            return;
        };
        let measured = popup.clone();
        observe_resize(&popup, move || {
            let auto = measured
                .style()
                .get_property_value("--popup-width")
                .is_ok_and(|width| width == "auto");
            if auto && context.open_now() {
                let (width, height) = size(&measured);
                if let Some(positioner) = context.positioner() {
                    set_sizes(&positioner, "--positioner", Some((width, height)));
                }
                context.place(width, height, false);
            }
        });
    });
    let starting = leptos::tachys::html::event::on(
        ev::Custom::<web_sys::ToggleEvent>::new("beforetoggle"),
        move |event: web_sys::ToggleEvent| {
            if event.new_state() == "open" {
                menu.phase.set(TransitionStatus::Starting);
                after_paint(move || {
                    if menu.phase.try_get_untracked() == Some(TransitionStatus::Starting) {
                        menu.phase.set(TransitionStatus::Idle);
                    }
                });
            }
        },
    );
    view! {
        <div
            {..attrs}
            {..starting}
            node_ref=menu.positioner
            popover="manual"
            role="presentation"
            style=POSITIONER_STYLE
            inert=move || !open()
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-side=move || menu.shown_side.get().as_str()
            data-align=move || menu.shown_align.get().as_str()
            data-instant=move || menu.instant.get().then_some("")
            on:toggle=move |event: web_sys::Event| {
                let shown = event.unchecked_ref::<web_sys::ToggleEvent>().new_state() == "open";
                menu.mounted.set(shown);
                if !shown {
                    context.closed();
                }
            }
            on:mouseenter=move |_| menu.hover.hold()
            on:mouseleave=move |_| {
                if !menu.touch.get_value() {
                    menu.hover.leave(
                        menu.close_delay.get_untracked(),
                        Callback::new(move |()| context.close(ChangeReason::TriggerHover)),
                    );
                }
            }
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "nav", skip(id, tabindex))]
pub fn NavigationMenuPopup(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<NavigationMenuContext>();
    let menu = context.menu();
    let open = move || context.open();
    let origin = move || {
        let side = menu.shown_side.get();
        let rtl = menu.direction.get() == Direction::Rtl;
        let left = side == Side::Left
            || (rtl && side == Side::InlineEnd)
            || (!rtl && side == Side::InlineStart);
        (side == Side::Top || left).then_some((side == Side::Top, left))
    };
    view! {
        <nav
            {..attrs}
            {..transition_attributes(menu.phase)}
            node_ref=menu.popup
            id=move || menu.popup_id.get()
            tabindex="-1"
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-side=move || menu.shown_side.get().as_str()
            data-align=move || menu.shown_align.get().as_str()
            style:transition=move || (menu.phase.get() == TransitionStatus::Starting).then_some("none")
            style:position=move || origin().map(|_| "absolute")
            style:bottom=move || origin().and_then(|(top, _)| top.then_some("0"))
            style:top=move || origin().and_then(|(top, _)| (!top).then_some("0"))
            style:right=move || origin().and_then(|(_, left)| left.then_some("0"))
            style:left=move || origin().and_then(|(_, left)| (!left).then_some("0"))
        >
            {inside_guard(context, true)}
            {children.map(|children| children())}
            {inside_guard(context, false)}
        </nav>
    }
}

fn inside_guard(context: NavigationMenuContext, before: bool) -> impl IntoView {
    let menu = context.menu();
    let node_ref = if before {
        menu.before_inside
    } else {
        menu.after_inside
    };
    view! {
        <span
            node_ref=node_ref
            aria-hidden="true"
            tabindex="0"
            data-base-ui-focus-guard=""
            style=GUARD_STYLE
            on:focus=move |event: ev::FocusEvent| {
                let from_outside = !context
                    .positioner()
                    .is_some_and(|positioner| positioner.contains(related(&event).as_ref()));
                if from_outside {
                    if let Some(target) = tabbable(if before { 1 } else { -1 }) {
                        let _ = target.focus();
                    }
                    return;
                }
                let guard = menu
                    .value
                    .get_untracked()
                    .and_then(|value| context.entry(&value))
                    .and_then(|entry| if before { entry.before } else { entry.after }.get_untracked());
                if let Some(guard) = guard {
                    let _ = guard.focus();
                }
            }
        ></span>
    }
}

#[component(element = "div", skip(id))]
pub fn NavigationMenuViewport(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let menu = expect_context::<NavigationMenuContext>().menu();
    view! {
        <div {..attrs} node_ref=menu.viewport id=move || menu.viewport_id.get()>
            {children.map(|children| children())}
        </div>
    }
}

fn js_sys_now() -> f64 {
    web_sys::js_sys::Date::now()
}
