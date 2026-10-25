use crate::anchor::{Placement, placement};
use crate::button::{button_attributes, composite_button_attributes};
use crate::composite::Composite;
use crate::dismiss::dismiss;
use crate::hover::Hover;
use crate::id::ElementId;
use crate::native::{keep_gutter, release_gutter};
use crate::{Controlled, Native, NativeOpen, reason, use_direction, use_id};
use leptos::context::Provider;
use leptos::either::Either;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_kit_macros::component;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use web_sys::{FocusOptions, HtmlElement, Node};

const SUBMENU_DELAY: u32 = 100;
const TYPEAHEAD_RESET: u32 = 500;
// a press this soon after a hover opened the menu leaves it open
const PATIENT_CLICK: f64 = 500.0;
// a release on an item this long after the press that opened the menu picks it
const DRAG_SELECT: u32 = 200;
const LONG_PRESS: u32 = 500;

static KEYS: AtomicUsize = AtomicUsize::new(0);

// what takes the focus once the popup shows
#[derive(Clone, Copy, PartialEq)]
enum Focus {
    Nothing,
    Popup,
    First,
    Last,
}

#[derive(Clone, Copy)]
struct Entry {
    id: usize,
    element: NodeRef<html::Div>,
    label: Option<Signal<String>>,
    // the menu a submenu trigger opens
    submenu: Option<usize>,
}

#[derive(Clone, Copy)]
struct Typeahead {
    text: StoredValue<String>,
    reset: StoredValue<Option<TimeoutHandle>>,
    matched: StoredValue<Option<usize>>,
}

struct Options {
    disabled: Signal<bool>,
    modal: Signal<bool>,
    orientation: Signal<&'static str>,
    loop_focus: Signal<bool>,
    highlight_on_hover: Signal<bool>,
    close_parent_on_esc: Signal<bool>,
    context_menu: bool,
    bar: Option<Bar>,
}

#[derive(Clone, Copy)]
pub(crate) struct MenuContext {
    key: usize,
    id: ElementId<html::Div>,
    trigger_fallback: StoredValue<String>,
    native: NativeOpen<html::Div>,
    positioner: NodeRef<html::Div>,
    popup: NodeRef<html::Div>,
    backdrop: NodeRef<html::Div>,
    button: NodeRef<html::Button>,
    row: NodeRef<html::Div>,
    parent: Option<StoredValue<MenuContext>>,
    disabled: Signal<bool>,
    modal: Signal<bool>,
    orientation: Signal<&'static str>,
    loop_focus: Signal<bool>,
    highlight_on_hover: Signal<bool>,
    close_parent_on_esc: Signal<bool>,
    direction: Signal<&'static str>,
    entries: StoredValue<Vec<Entry>>,
    next: StoredValue<usize>,
    highlighted: RwSignal<Option<usize>>,
    // the submenu open from this menu, one at a time
    child: StoredValue<Option<MenuContext>>,
    hover: Hover,
    // base ui's hoverEnabled: once the pointer works inside the popup, leaving it no longer closes it
    hoverable: StoredValue<bool>,
    close_delay: StoredValue<u32>,
    focus: StoredValue<Focus>,
    opened: StoredValue<(&'static str, f64)>,
    drag: StoredValue<bool>,
    typeahead: Typeahead,
    // a context menu's anchor: the point it opened at, its size, and the element there
    point: Option<RwSignal<(f64, f64, f64)>>,
    spot: NodeRef<html::Span>,
    // a context menu has no trigger to go back to, so the focus returns where it was
    before: StoredValue<Option<HtmlElement>, LocalStorage>,
    bar: Option<Bar>,
}

impl MenuContext {
    fn new(state: Controlled<bool>, parent: Option<MenuContext>, options: Options) -> Self {
        let positioner = NodeRef::new();
        let popup = NodeRef::new();
        let id = ElementId::new(positioner, use_id());
        let context = Self {
            key: KEYS.fetch_add(1, Ordering::Relaxed),
            id,
            trigger_fallback: StoredValue::new(format!("{}-trigger", id.get())),
            native: NativeOpen::new(Native::Popover, positioner, state, Some(popup)),
            positioner,
            popup,
            backdrop: NodeRef::new(),
            button: NodeRef::new(),
            row: NodeRef::new(),
            parent: parent.map(StoredValue::new),
            disabled: options.disabled,
            modal: options.modal,
            orientation: options.orientation,
            loop_focus: options.loop_focus,
            highlight_on_hover: options.highlight_on_hover,
            close_parent_on_esc: options.close_parent_on_esc,
            direction: use_direction(),
            entries: StoredValue::new(Vec::new()),
            next: StoredValue::new(0),
            highlighted: RwSignal::new(None),
            child: StoredValue::new(None),
            hover: Hover::new(),
            hoverable: StoredValue::new(true),
            close_delay: StoredValue::new(0),
            focus: StoredValue::new(Focus::Nothing),
            opened: StoredValue::new((reason::NONE, f64::NEG_INFINITY)),
            drag: StoredValue::new(false),
            typeahead: Typeahead {
                text: StoredValue::new(String::new()),
                reset: StoredValue::new(None),
                matched: StoredValue::new(None),
            },
            point: options.context_menu.then(|| RwSignal::new((0.0, 0.0, 0.0))),
            spot: NodeRef::new(),
            before: StoredValue::new_local(None),
            bar: options.bar,
        };
        if let Some(bar) = options.bar {
            bar.menus.update_value(|menus| menus.push(context));
            on_cleanup(move || {
                bar.menus
                    .try_update_value(|menus| menus.retain(|menu| menu.key != context.key));
            });
        }
        on_cleanup(move || {
            if let Some(Some(handle)) = context.typeahead.reset.try_update_value(Option::take) {
                handle.clear();
            }
            context.unlock();
        });
        context
    }

    fn open(&self) -> bool {
        self.native.state().get()
    }

    fn parent(&self) -> Option<MenuContext> {
        self.parent.and_then(|parent| parent.try_get_value())
    }

    fn root(self) -> Self {
        let mut menu = self;
        while let Some(parent) = menu.parent() {
            menu = parent;
        }
        menu
    }

    fn anchor(&self) -> String {
        format!("--{}", self.id.get())
    }

    fn popup_id(&self) -> String {
        format!("{}-popup", self.id.get())
    }

    // once the trigger is in the dom its own id wins, the client's count can drift from the server's
    fn trigger_id(&self) -> String {
        self.trigger()
            .map(|trigger| trigger.id())
            .filter(|id| !id.is_empty())
            .or_else(|| self.trigger_fallback.try_get_value())
            .unwrap_or_default()
    }

    fn trigger(&self) -> Option<HtmlElement> {
        let button = self
            .button
            .try_get_untracked()
            .flatten()
            .map(HtmlElement::from);
        button.or_else(|| {
            self.row
                .try_get_untracked()
                .flatten()
                .map(HtmlElement::from)
        })
    }

    fn anchored(&self) -> Option<web_sys::Element> {
        let spot = self.spot.try_get_untracked().flatten();
        self.trigger()
            .map(Into::into)
            .or_else(|| spot.map(Into::into))
    }

    // its trigger and its positioner, which holds its submenus too
    fn contains(&self, node: &Node) -> bool {
        let positioner = self
            .positioner
            .try_get_untracked()
            .flatten()
            .map(Node::from);
        [self.trigger().map(Node::from), positioner]
            .iter()
            .flatten()
            .any(|part| part.contains(Some(node)))
    }

    fn show(self, reason: &'static str, focus: Focus) {
        if self.disabled.get_untracked() || self.native.state().get_untracked() {
            return;
        }
        self.focus.set_value(focus);
        self.opened.set_value((reason, now()));
        self.hoverable.set_value(true);
        if self.point.is_some() {
            let active = document().active_element();
            self.before
                .set_value(active.and_then(|active| active.dyn_into::<HtmlElement>().ok()));
        }
        if let Some(parent) = self.parent() {
            let previous = parent
                .child
                .try_update_value(|child| child.replace(self))
                .flatten();
            if let Some(previous) = previous
                && previous.key != self.key
            {
                previous.hide(reason::SIBLING_OPEN);
            }
        }
        if let Some(bar) = self.bar {
            let previous = bar
                .open
                .try_update_value(|open| open.replace(self))
                .flatten();
            if let Some(previous) = previous
                && previous.key != self.key
            {
                previous.hide(reason::SIBLING_OPEN);
            }
            bar.has_open.set(true);
        }
        self.native.state().set(true, reason);
        // a menubar stays modal whichever way its menus open, as base ui's
        let modal = match self.bar {
            Some(bar) => bar.modal.get_untracked(),
            None => {
                self.parent.is_none()
                    && self.modal.get_untracked()
                    && reason != reason::TRIGGER_HOVER
            }
        };
        if modal && self.native.state().get_untracked() {
            self.lock();
        }
    }

    fn hide(self, reason: &'static str) {
        if !self.native.state().get_untracked() {
            return;
        }
        if let Some(Some(child)) = self.child.try_update_value(Option::take) {
            child.hide(reason);
        }
        self.native.state().set(false, reason);
        self.closed();
        self.left_bar(reason);
        // the focus is on its way somewhere else
        if reason != reason::FOCUS_OUT {
            self.return_focus();
        }
    }

    // once it's closed, whether it was asked to or the browser did it
    fn closed(self) {
        self.hover.cancel();
        self.hoverable.set_value(true);
        self.highlighted.try_set(None);
        if let Some(parent) = self.parent() {
            parent.child.try_update_value(|child| {
                if child.is_some_and(|child| child.key == self.key) {
                    *child = None;
                }
            });
        }
        self.unlock();
    }

    // a menubar keeps opening its menus on the way until one closes for good
    fn left_bar(self, reason: &'static str) {
        let Some(bar) = self.bar else {
            return;
        };
        bar.open.try_update_value(|open| {
            if open.is_some_and(|open| open.key == self.key) {
                *open = None;
            }
        });
        if reason != reason::SIBLING_OPEN && reason != reason::LIST_NAVIGATION {
            bar.has_open.try_set(false);
        }
    }

    // the focus goes back to the trigger unless it already moved somewhere else
    fn return_focus(self) {
        let back = self
            .trigger()
            .or_else(|| self.before.try_get_value().flatten());
        let (Some(positioner), Some(trigger)) =
            (self.positioner.try_get_untracked().flatten(), back)
        else {
            return;
        };
        let stranded = match document().active_element() {
            Some(active) => {
                positioner.contains(Some(&active)) || active.tag_name().eq_ignore_ascii_case("body")
            }
            None => true,
        };
        if stranded {
            focus(&trigger);
        }
    }

    // base ui's modal menu: the page can't scroll, a press outside only closes it
    fn lock(self) {
        let Some(backdrop) = self.backdrop.try_get_untracked().flatten() else {
            return;
        };
        keep_gutter();
        // the trigger, or the whole menubar, stays reachable through it, as base ui's cutout
        let bar = self
            .bar
            .and_then(|bar| bar.element.try_get_untracked().flatten())
            .map(HtmlElement::from);
        if let Some(trigger) = bar.or_else(|| self.trigger()) {
            let r = trigger.get_bounding_client_rect();
            let (left, top, right, bottom) = (r.left(), r.top(), r.right(), r.bottom());
            let clip = format!(
                "polygon(0% 0%,100% 0%,100% 100%,0% 100%,0% 0%,{left}px {top}px,{left}px {bottom}px,\
                 {right}px {bottom}px,{right}px {top}px,{left}px {top}px)"
            );
            let _ = HtmlElement::style(&backdrop).set_property("clip-path", &clip);
        }
        let _ = backdrop.show_popover();
    }

    fn unlock(self) {
        let Some(backdrop) = self.backdrop.try_get_untracked().flatten() else {
            return;
        };
        if backdrop.matches(":popover-open").unwrap_or(false) {
            let _ = backdrop.hide_popover();
            release_gutter(Some(backdrop.into()));
        }
    }

    fn ordered(&self) -> Vec<(Entry, HtmlElement)> {
        let mut items: Vec<(Entry, HtmlElement)> = self
            .entries
            .try_with_value(|entries| {
                entries
                    .iter()
                    .filter_map(|entry| {
                        let element = entry.element.try_get_untracked().flatten()?;
                        Some((*entry, element.into()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        items.sort_by(|(_, a), (_, b)| {
            if a.compare_document_position(b) & Node::DOCUMENT_POSITION_FOLLOWING != 0 {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });
        items
    }

    // the highlight is the focus, as base ui's menus
    fn highlight(self, id: Option<usize>) {
        self.highlighted.set(id);
        let item = id.and_then(|id| {
            self.ordered()
                .into_iter()
                .find(|(entry, _)| entry.id == id)
                .map(|(_, element)| element)
        });
        let target = item.or_else(|| self.popup.get_untracked().map(HtmlElement::from));
        if let Some(target) = target {
            focus(&target);
        }
    }

    fn highlight_end(self, last: bool) {
        let items = self.ordered();
        let end = if last { items.last() } else { items.first() };
        if let Some((entry, _)) = end {
            self.highlight(Some(entry.id));
        }
    }

    fn take_focus(self) {
        match self.focus.get_value() {
            Focus::First => self.highlight_end(false),
            Focus::Last => self.highlight_end(true),
            Focus::Popup => self.highlight(None),
            Focus::Nothing => {}
        }
        self.focus.set_value(Focus::Nothing);
    }

    fn hovered(self, id: usize, event: &ev::PointerEvent) {
        if event.pointer_type() == "touch" {
            return;
        }
        // a submenu stays only while its own trigger is under the pointer
        let submenu = self
            .entries
            .try_with_value(|entries| {
                entries
                    .iter()
                    .find(|entry| entry.id == id)
                    .and_then(|entry| entry.submenu)
            })
            .flatten();
        if let Some(Some(child)) = self.child.try_get_value()
            && submenu != Some(child.key)
        {
            child.hide(reason::SIBLING_OPEN);
        }
        if self.highlight_on_hover.get_untracked() && self.highlighted.get_untracked() != Some(id) {
            self.highlight(Some(id));
        }
    }

    fn unhovered(self, event: &ev::PointerEvent) {
        if event.pointer_type() == "touch" || !self.highlight_on_hover.get_untracked() {
            return;
        }
        let items = self.ordered();
        let to = event
            .related_target()
            .and_then(|target| target.dyn_into::<Node>().ok());
        if to.is_some_and(|to| items.iter().any(|(_, element)| element.contains(Some(&to)))) {
            return;
        }
        // only when the focus is on one of this menu's items, not in a submenu
        let focused = document().active_element();
        let own = focused.is_some_and(|focused| {
            items
                .iter()
                .any(|(_, element)| element.is_same_node(Some(&focused)))
        });
        if own {
            self.highlight(None);
        } else {
            self.highlighted.set(None);
        }
    }

    fn keydown(self, event: &ev::KeyboardEvent) {
        let key = event.key();
        if key == "Escape" {
            stop(event);
            // esc closes the deepest open submenu, wherever the focus is, as base ui's
            let mut target = self;
            while let Some(Some(child)) = target.child.try_get_value() {
                target = child;
            }
            if target.close_parent_on_esc.get_untracked() {
                target = target.root();
            }
            target.hide(reason::ESCAPE_KEY);
            return;
        }
        // in a menubar the keys across move to the next menu
        if let Some(bar) = self.bar
            && bar.orientation.get_untracked() != "vertical"
        {
            let rtl = self.direction.get_untracked() == "rtl";
            let (next, previous) = if rtl {
                ("ArrowLeft", "ArrowRight")
            } else {
                ("ArrowRight", "ArrowLeft")
            };
            if key == next || key == previous {
                stop(event);
                bar.step(self, key == next);
                return;
            }
        }
        let vertical = self.orientation.get_untracked() != "horizontal";
        let rtl = self.direction.get_untracked() == "rtl";
        let (forward, backward) = match (vertical, rtl) {
            (true, _) => ("ArrowDown", "ArrowUp"),
            (false, false) => ("ArrowRight", "ArrowLeft"),
            (false, true) => ("ArrowLeft", "ArrowRight"),
        };
        let back = if rtl { "ArrowRight" } else { "ArrowLeft" };
        if self.parent.is_some() && vertical && key == back {
            stop(event);
            self.hide(reason::LIST_NAVIGATION);
            if let Some(trigger) = self.trigger() {
                focus(&trigger);
            }
            return;
        }
        if key == "Tab" && event.shift_key() {
            stop(event);
            self.hide(reason::FOCUS_OUT);
            self.return_focus();
            return;
        }
        // a context menu keeps the focus, as base ui's modal focus manager
        if key == "Tab" && self.root().point.is_some() {
            event.prevent_default();
            return;
        }
        if key == forward || key == backward || key == "Home" || key == "End" {
            stop(event);
            self.navigate(&key, key == forward);
            return;
        }
        if self.type_ahead(&key, event) {
            stop(event);
        }
    }

    fn navigate(self, key: &str, forward: bool) {
        let items = self.ordered();
        let (Some(first), Some(last)) = (items.first(), items.last()) else {
            return;
        };
        let at = self
            .highlighted
            .get_untracked()
            .and_then(|id| items.iter().position(|(entry, _)| entry.id == id));
        let next = match (key, at) {
            ("Home", _) => first.0.id,
            ("End", _) => last.0.id,
            (_, None) if forward => first.0.id,
            (_, None) => last.0.id,
            (_, Some(at)) => {
                let looping = self.loop_focus.get_untracked();
                let index = if forward {
                    match at + 1 < items.len() {
                        true => at + 1,
                        false if looping => 0,
                        false => at,
                    }
                } else {
                    match at > 0 {
                        true => at - 1,
                        false if looping => items.len() - 1,
                        false => at,
                    }
                };
                items[index].0.id
            }
        };
        self.highlight(Some(next));
    }

    // base ui's typeahead: the typed letters pick the next item whose text starts with them
    fn type_ahead(self, key: &str, event: &ev::KeyboardEvent) -> bool {
        let typeahead = self.typeahead;
        let typing = !typeahead.text.get_value().is_empty();
        if key.chars().count() != 1 || event.ctrl_key() || event.meta_key() || event.alt_key() {
            return false;
        }
        if key == " " && !typing {
            return false;
        }
        let items = self.ordered();
        let labels: Vec<String> = items
            .iter()
            .map(|(entry, element)| {
                entry
                    .label
                    .map(|label| label.get_untracked())
                    .or_else(|| element.text_content())
                    .unwrap_or_default()
                    .to_lowercase()
            })
            .collect();
        if labels.is_empty() {
            return false;
        }
        let at = self
            .highlighted
            .get_untracked()
            .and_then(|id| items.iter().position(|(entry, _)| entry.id == id));
        let mut previous = if typing {
            typeahead.matched.get_value()
        } else {
            at
        };
        let key = key.to_lowercase();
        // repeating one letter cycles through the items that start with it
        let cycles = labels.iter().all(|label| {
            let mut chars = label.chars();
            match (chars.next(), chars.next()) {
                (Some(first), Some(second)) => first != second,
                _ => true,
            }
        });
        if cycles && typeahead.text.get_value() == key {
            typeahead.text.set_value(String::new());
            previous = typeahead.matched.get_value();
        }
        typeahead.text.update_value(|text| text.push_str(&key));
        if let Some(Some(handle)) = typeahead.reset.try_update_value(Option::take) {
            handle.clear();
        }
        let reset = set_timeout_with_handle(
            move || {
                typeahead.text.try_set_value(String::new());
            },
            Duration::from_millis(TYPEAHEAD_RESET.into()),
        );
        typeahead.reset.set_value(reset.ok());
        let text = typeahead.text.get_value();
        let start = previous.map_or(0, |previous| previous + 1);
        let found = (0..labels.len())
            .map(|offset| (start + offset) % labels.len())
            .find(|&index| labels[index].starts_with(&text));
        match found {
            Some(index) => {
                typeahead.matched.set_value(Some(index));
                self.highlight(Some(items[index].0.id));
            }
            None if key != " " => typeahead.text.set_value(String::new()),
            None => {}
        }
        true
    }

    fn typing(&self) -> bool {
        !self.typeahead.text.get_value().is_empty()
    }
}

fn now() -> f64 {
    web_sys::js_sys::Date::now()
}

fn focus(element: &HtmlElement) {
    let options = FocusOptions::new();
    options.set_prevent_scroll(true);
    let _ = element.focus_with_options(&options);
}

fn stop(event: &ev::KeyboardEvent) {
    event.prevent_default();
    event.stop_propagation();
}

// an auto popover: the browser light-dismisses it, nests its submenus and keeps one open a level
#[leptos::component]
pub fn MenuRoot(
    #[prop(optional, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, &'static str)>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(into, default = true.into())] modal: Signal<bool>,
    #[prop(into, default = true.into())] loop_focus: Signal<bool>,
    #[prop(into, default = "vertical".into())] orientation: Signal<&'static str>,
    #[prop(into, default = true.into())] highlight_item_on_hover: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let state = Controlled::new(open, default_open, on_open_change);
    let options = Options {
        disabled,
        modal,
        orientation,
        loop_focus,
        highlight_on_hover: highlight_item_on_hover,
        close_parent_on_esc: false.into(),
        context_menu: false,
        bar: use_context::<Bar>(),
    };
    let context = MenuContext::new(state, None, options);
    // components share their parent's owner, a sibling root would overwrite the context
    view! { <Provider value=context>{children()}</Provider> }
}

#[leptos::component]
pub fn MenuSubmenuRoot(
    #[prop(optional, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, &'static str)>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(into, default = true.into())] loop_focus: Signal<bool>,
    #[prop(into, default = "vertical".into())] orientation: Signal<&'static str>,
    #[prop(into, default = true.into())] highlight_item_on_hover: Signal<bool>,
    #[prop(optional, into)] close_parent_on_esc: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let parent = expect_context::<MenuContext>();
    let state = Controlled::new(open, default_open, on_open_change);
    let options = Options {
        disabled: Signal::derive(move || disabled.get() || parent.disabled.get()),
        modal: false.into(),
        orientation,
        loop_focus,
        highlight_on_hover: highlight_item_on_hover,
        close_parent_on_esc,
        context_menu: false,
        bar: None,
    };
    let context = MenuContext::new(state, Some(parent), options);
    view! { <Provider value=context>{children()}</Provider> }
}

#[component(
    element = "button",
    skip(r#type, id, role, popovertarget, popovertargetaction)
)]
pub fn MenuTrigger(
    #[prop(optional_no_strip, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional_no_strip, into)] id: Option<String>,
    #[prop(optional, into)] open_on_hover: Signal<bool>,
    #[prop(into, default = 100.into())] delay: Signal<u32>,
    #[prop(optional, into)] close_delay: Signal<u32>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<MenuContext>();
    if let Some(id) = id {
        context.trigger_fallback.set_value(id);
    }
    let bar = context.bar;
    let disabled = move || {
        context.disabled.get()
            || disabled.is_some_and(|disabled| disabled.get())
            || bar.is_some_and(|bar| bar.disabled.get())
    };
    let open = move || context.open();
    let mouse = |event: &ev::PointerEvent| event.pointer_type() == "mouse";
    // in a menubar it's one of the items its arrows move between
    let item = bar.map(|bar| {
        let id = bar
            .composite
            .item(Signal::derive(disabled), false.into(), context.button);
        (bar, id)
    });
    let attributes = match item {
        Some((bar, id)) => Either::Left(composite_button_attributes(
            disabled,
            bar.composite.tabindex(id),
        )),
        None => Either::Right(button_attributes(disabled, || false, tabindex)),
    };
    let release = StoredValue::new(None::<WindowListenerHandle>);
    let arm = StoredValue::new(None::<TimeoutHandle>);
    // a press that opened it, released over an item, picks it; released elsewhere, closes it
    let released = move |event: ev::MouseEvent| {
        if let Some(Some(handle)) = release.try_update_value(Option::take) {
            handle.remove();
        }
        if let Some(Some(handle)) = arm.try_update_value(Option::take) {
            handle.clear();
        }
        context.drag.set_value(false);
        let target = event
            .target()
            .and_then(|target| target.dyn_into::<Node>().ok());
        if target.is_some_and(|target| context.contains(&target)) {
            return;
        }
        let within = context.trigger().is_some_and(|trigger| {
            let r = trigger.get_bounding_client_rect();
            let (x, y) = (f64::from(event.client_x()), f64::from(event.client_y()));
            x >= r.left() && x <= r.right() && y >= r.top() && y <= r.bottom()
        });
        if !within {
            context.hide(reason::CANCEL_OPEN);
        }
    };

    view! {
        <button
            {..attrs}
            {..attributes}
            node_ref=context.button
            id=context.trigger_fallback.get_value()
            type="button"
            role=bar.is_some().then_some("menuitem")
            style:anchor-name=context.anchor()
            popovertarget=context.id.get()
            aria-haspopup="menu"
            aria-expanded=move || open().to_string()
            aria-controls=move || open().then(|| context.popup_id())
            data-popup-open=move || open().then_some("")
            data-pressed=move || open().then_some("")
            on:mousedown=move |event: ev::MouseEvent| {
                if event.button() != 0 || event.ctrl_key() || disabled() {
                    return;
                }
                if !context.native.state().get_untracked() {
                    context.show(reason::TRIGGER_PRESS, Focus::Popup);
                    let handle = set_timeout_with_handle(
                        move || context.drag.set_value(true),
                        Duration::from_millis(DRAG_SELECT.into()),
                    );
                    arm.set_value(handle.ok());
                    release.set_value(Some(window_event_listener(ev::mouseup, released)));
                    return;
                }
                let (why, at) = context.opened.get_value();
                if why != reason::TRIGGER_HOVER || now() - at > PATIENT_CLICK {
                    context.hide(reason::TRIGGER_PRESS);
                }
            }
            on:click=move |event: ev::MouseEvent| {
                // the press already did it, and the browser's own toggle would undo it
                event.prevent_default();
                if event.detail() != 0 || disabled() {
                    return;
                }
                if context.native.state().get_untracked() {
                    context.hide(reason::TRIGGER_PRESS);
                } else {
                    context.show(reason::TRIGGER_PRESS, Focus::First);
                }
            }
            on:keydown=move |event: ev::KeyboardEvent| {
                let key = event.key();
                // a menubar opens its menus with the key across it, onto their first item
                let (down, up) = match bar.map(|bar| bar.orientation.get_untracked()) {
                    Some("vertical") if context.direction.get_untracked() == "rtl" => ("ArrowLeft", ""),
                    Some("vertical") => ("ArrowRight", ""),
                    Some(_) => ("ArrowDown", ""),
                    None => ("ArrowDown", "ArrowUp"),
                };
                if disabled() || key.is_empty() || (key != down && key != up) {
                    return;
                }
                event.prevent_default();
                let last = key == up;
                if context.native.state().get_untracked() {
                    context.highlight_end(last);
                } else {
                    context
                        .show(reason::LIST_NAVIGATION, if last { Focus::Last } else { Focus::First });
                }
            }
            on:focus=move |event: ev::FocusEvent| {
                let Some((bar, id)) = item else {
                    return;
                };
                bar.composite.highlight(id);
                let visible = event_target::<HtmlElement>(&event)
                    .matches(":focus-visible")
                    .unwrap_or(false);
                let moving = bar.has_open.get_untracked() && !bar.switching.get_value();
                if visible && moving && !context.native.state().get_untracked() {
                    context.show(reason::TRIGGER_FOCUS, Focus::Popup);
                }
            }
            on:pointerenter=move |event: ev::PointerEvent| {
                if let Some(bar) = bar {
                    let moving = bar.has_open.get_untracked() && !context.native.state().get_untracked();
                    if mouse(&event) && moving && !disabled() {
                        bar.switch_to(context, reason::TRIGGER_HOVER);
                    }
                    return;
                }
                if mouse(&event) && open_on_hover.get_untracked() && !disabled() {
                    context.close_delay.set_value(close_delay.get_untracked());
                    let open = Callback::new(move |_| context.show(reason::TRIGGER_HOVER, Focus::Nothing));
                    context.hover.enter(delay.get_untracked(), open);
                }
            }
            on:pointerleave=move |event: ev::PointerEvent| {
                if mouse(&event) && open_on_hover.get_untracked() {
                    leave(context, &event);
                }
            }
        >
            {children.map(|children| children())}
        </button>
    }
}

// leaving a trigger keeps its menu open while the pointer heads for the popup
fn leave(context: MenuContext, event: &ev::PointerEvent) {
    let close = Callback::new(move |_| context.hide(reason::TRIGGER_HOVER));
    let delay = context.close_delay.get_value();
    let boxes = context.trigger().zip(context.popup.get_untracked());
    match boxes {
        Some((trigger, popup)) if context.native.state().get_untracked() => {
            if !context.hoverable.get_value() {
                return;
            }
            let from = (f64::from(event.client_x()), f64::from(event.client_y()));
            context
                .hover
                .cross(from, [trigger.into(), popup.into()], delay, close);
        }
        _ => context.hover.leave(delay, close),
    }
}

#[leptos::component]
pub fn MenuPortal(children: Children) -> impl IntoView {
    children()
}

#[component(element = "div", skip(id, style, role))]
pub fn MenuPositioner(
    #[prop(optional, into)] side: Option<Signal<&'static str>>,
    #[prop(optional, into)] align: Option<Signal<&'static str>>,
    #[prop(optional, into)] side_offset: Option<Signal<f64>>,
    #[prop(optional, into)] align_offset: Option<Signal<f64>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<MenuContext>();
    let nested = context.parent.is_some();
    let point = context.point;
    let bar = context.bar;
    // a submenu opens beside its trigger, a context menu from the corner at the pointer, as base ui's
    let at_corner = point.is_some() && side.is_none();
    let side = Signal::derive(move || {
        let across = bar.is_some_and(|bar| bar.orientation.get() == "vertical");
        side.map_or(
            if nested || across {
                "inline-end"
            } else {
                "bottom"
            },
            |side| side.get(),
        )
    });
    let align = Signal::derive(move || {
        let start = nested || point.is_some() || bar.is_some();
        align.map_or(if start { "start" } else { "center" }, |align| align.get())
    });
    let offset = move |offset: Option<Signal<f64>>, corner: f64| {
        move || match offset {
            Some(offset) => offset.get(),
            None if at_corner && align.get() != "center" => corner,
            None => 0.0,
        }
    };
    let (side_offset, align_offset) = (offset(side_offset, -5.0), offset(align_offset, 2.0));
    let placed = Placement::new(side, align);
    let open = move || context.open();
    let style = move || {
        #[cfg(debug_assertions)]
        crate::anchor::check("MenuPositioner", side.get(), align.get());
        let place = placement(
            &context.anchor(),
            side.get(),
            align.get(),
            side_offset(),
            align_offset(),
        );
        let inert = if open() { "" } else { "pointer-events:none" };
        format!("{place};{};{inert}", (placed.sizes())())
    };
    // show_popover runs in the root's effect, so the boxes are in place a microtask later
    Effect::new(move || {
        if context.native.state().get() {
            queue_microtask(move || {
                if let (Some(anchor), Some(Some(positioner))) =
                    (context.anchored(), context.positioner.try_get_untracked())
                {
                    placed.settle(&anchor, &positioner);
                }
            });
            // after the press that opened it has focused the trigger
            request_animation_frame(move || context.take_focus());
        } else {
            placed.flipped.try_set(false);
        }
    });
    let backdrop = (!nested).then(|| {
        view! {
            <div
                node_ref=context.backdrop
                popover="manual"
                role="presentation"
                data-modal-backdrop=""
                style="position:fixed;inset:0;width:auto;height:auto;max-width:none;max-height:none;margin:0;padding:0;border:0;background:transparent;user-select:none;-webkit-user-select:none"
                on:contextmenu=move |event: ev::MouseEvent| {
                    if point.is_some() {
                        event.prevent_default();
                    }
                }
            ></div>
        }
    });
    let spot = point.map(|point| {
        let at = move |pick: fn((f64, f64, f64)) -> f64| move || format!("{}px", pick(point.get()));
        view! {
            <span
                node_ref=context.spot
                aria-hidden="true"
                style="position:fixed;pointer-events:none"
                style:left=at(|(x, _, _)| x)
                style:top=at(|(_, y, _)| y)
                style:width=at(|(_, _, size)| size)
                style:height=at(|(_, _, size)| size)
                style:anchor-name=context.anchor()
            ></span>
        }
    });
    // the browser's light dismiss would close a context menu on the release of the press that opened it
    let kind = if point.is_some() { "manual" } else { "auto" };
    view! {
        {backdrop}
        {spot}
        <div
            {..attrs}
            node_ref=context.positioner
            id=context.id.get()
            popover=kind
            role="presentation"
            style=style
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-side=move || placed.side()
            data-align=align
            data-nested=nested.then_some("")
            // only the browser's own close is news, ours already tidied up and may have reopened since
            on:toggle=move |_| {
                let was = context.native.state().try_get_untracked() == Some(true);
                context.native.sync(reason::OUTSIDE_PRESS);
                if was && context.native.state().try_get_untracked() == Some(false) {
                    context.closed();
                    context.left_bar(reason::OUTSIDE_PRESS);
                    context.return_focus();
                }
            }
        >
            <Provider value=placed>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "div", skip(id, role, tabindex))]
pub fn MenuPopup(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<MenuContext>();
    let placed = expect_context::<Placement>();
    let open = move || context.open();
    let nested = context.parent.is_some();
    view! {
        <div
            {..attrs}
            node_ref=context.popup
            id=context.popup_id()
            role="menu"
            tabindex="-1"
            aria-labelledby=move || context.point.is_none().then(|| context.trigger_id())
            aria-orientation=move || (context.orientation.get() == "horizontal").then_some("horizontal")
            style:position="relative"
            style:left=placed.nudge(|(x, _)| x)
            style:top=placed.nudge(|(_, y)| y)
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-side=move || placed.side()
            data-align=placed.align
            data-nested=nested.then_some("")
            on:keydown=move |event: ev::KeyboardEvent| context.keydown(&event)
            on:focusout=move |event: ev::FocusEvent| {
                if context.bar.is_some_and(|bar| bar.switching.get_value()) {
                    return;
                }
                let to = event.related_target().and_then(|target| target.dyn_into::<Node>().ok());
                if to.is_some_and(|to| !context.contains(&to)) {
                    context.hide(reason::FOCUS_OUT);
                }
            }
            on:pointerenter=move |_| context.hover.hold()
            on:pointermove=move |_| {
                if nested && context.native.state().get_untracked() {
                    context.hoverable.set_value(false);
                }
            }
            on:pointerleave=move |event: ev::PointerEvent| {
                if event.pointer_type() == "mouse"
                    && context.bar.is_none()
                    && context.opened.get_value().0 == reason::TRIGGER_HOVER
                {
                    leave(context, &event);
                }
            }
            // the press on an item that closed it reaches here after, when it no longer matters
            on:click=move |_| {
                if context.native.state().get_untracked() {
                    context.hoverable.set_value(false);
                }
            }
        >
            {children.map(|children| children())}
        </div>
    }
}

// what every item shares: a place in the list, the highlight, and the pointer over it
fn register(
    context: MenuContext,
    element: NodeRef<html::Div>,
    label: Option<Signal<String>>,
    submenu: Option<usize>,
) -> usize {
    let id = context
        .next
        .try_update_value(|next| {
            *next += 1;
            *next - 1
        })
        .unwrap_or_default();
    context.entries.update_value(|entries| {
        entries.push(Entry {
            id,
            element,
            label,
            submenu,
        })
    });
    on_cleanup(move || {
        context
            .entries
            .try_update_value(|entries| entries.retain(|entry| entry.id != id));
    });
    id
}

fn item_attributes(
    context: MenuContext,
    id: usize,
    disabled: Signal<bool>,
    tab_stop: impl Fn() -> bool + Copy + Send + Sync + 'static,
) -> impl leptos::attr::Attribute {
    let highlighted = move || context.highlighted.get() == Some(id);
    view! {
        <{..}
            tabindex=move || if (context.open() && highlighted()) || tab_stop() { 0 } else { -1 }
            aria-disabled=move || disabled.get().then_some("true")
            data-highlighted=move || highlighted().then_some("")
            data-disabled=move || disabled.get().then_some("")
            on:focus=move |_| context.highlighted.set(Some(id))
            on:pointermove=move |event: ev::PointerEvent| context.hovered(id, &event)
            on:pointerleave=move |event: ev::PointerEvent| context.unhovered(&event)
        />
    }
}

// enter and space press an item, a release after dragging from the trigger does too
fn press_attributes(
    context: MenuContext,
    element: NodeRef<html::Div>,
    disabled: Signal<bool>,
    close_on_click: Signal<bool>,
    action: impl Fn() + Copy + Send + Sync + 'static,
) -> impl leptos::attr::Attribute {
    let click = leptos::tachys::html::event::on(
        // not delegated, so it runs before the caller's on:click
        ev::Custom::<ev::MouseEvent>::new("click"),
        move |event: ev::MouseEvent| {
            if disabled.get_untracked() {
                event.prevent_default();
                event.stop_immediate_propagation();
                return;
            }
            action();
            if close_on_click.get_untracked() {
                context.root().hide(reason::ITEM_PRESS);
            }
        },
    );
    let keys = view! {
        <{..}
            on:keydown=move |event: ev::KeyboardEvent| {
                let key = event.key();
                if (key != "Enter" && key != " ") || disabled.get_untracked() {
                    return;
                }
                event.prevent_default();
                if key == " " && context.typing() {
                    return;
                }
                if let Some(element) = element.get_untracked() {
                    element.click();
                }
            }
            on:mouseup=move |_| {
                if context.root().drag.get_value()
                    && let Some(element) = element.get_untracked()
                {
                    element.click();
                }
            }
        />
    };
    (keys, click)
}

#[component(element = "div", skip(role, tabindex))]
pub fn MenuItem(
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(into, default = true.into())] close_on_click: Signal<bool>,
    #[prop(optional_no_strip, into)] label: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<MenuContext>();
    let element = NodeRef::new();
    let disabled = Signal::derive(move || disabled.get() || context.disabled.get());
    let id = register(context, element, label, None);
    view! {
        <div
            {..attrs}
            {..item_attributes(context, id, disabled, || false)}
            {..press_attributes(context, element, disabled, close_on_click, || {})}
            node_ref=element
            role="menuitem"
        >
            {children.map(|children| children())}
        </div>
    }
}

#[derive(Clone, Copy)]
struct Checked {
    checked: Signal<bool>,
    disabled: Signal<bool>,
    highlighted: Signal<bool>,
}

#[component(element = "div", skip(role, tabindex))]
pub fn MenuCheckboxItem(
    #[prop(optional_no_strip, into)] checked: Option<Signal<bool>>,
    #[prop(optional)] default_checked: bool,
    #[prop(optional_no_strip, into)] on_checked_change: Option<Callback<(bool, &'static str)>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] close_on_click: Signal<bool>,
    #[prop(optional_no_strip, into)] label: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<MenuContext>();
    let element = NodeRef::new();
    let disabled = Signal::derive(move || disabled.get() || context.disabled.get());
    let id = register(context, element, label, None);
    let state = Controlled::new(checked, default_checked, on_checked_change);
    let checked = Signal::from(state);
    let toggle = move || state.set(!state.get_untracked(), reason::ITEM_PRESS);
    let item = Checked {
        checked,
        disabled,
        highlighted: Signal::derive(move || context.highlighted.get() == Some(id)),
    };
    view! {
        <div
            {..attrs}
            {..item_attributes(context, id, disabled, || false)}
            {..press_attributes(context, element, disabled, close_on_click, toggle)}
            node_ref=element
            role="menuitemcheckbox"
            aria-checked=move || checked.get().to_string()
            data-checked=move || checked.get().then_some("")
            data-unchecked=move || (!checked.get()).then_some("")
        >
            <Provider value=item>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "span")]
pub fn MenuCheckboxItemIndicator(
    #[prop(optional)] keep_mounted: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let item = expect_context::<Checked>();
    indicator(item, keep_mounted, children)
}

fn indicator(item: Checked, keep_mounted: bool, children: Option<Children>) -> impl IntoView {
    let checked = item.checked;
    view! {
        <span
            aria-hidden="true"
            hidden=move || !keep_mounted && !checked.get()
            data-checked=move || checked.get().then_some("")
            data-unchecked=move || (!checked.get()).then_some("")
            data-disabled=move || item.disabled.get().then_some("")
            data-highlighted=move || item.highlighted.get().then_some("")
        >
            {children.map(|children| children())}
        </span>
    }
}

#[derive(Clone, Copy)]
struct RadioContext {
    value: Controlled<Option<String>>,
    disabled: Signal<bool>,
}

#[component(element = "div", skip(role))]
pub fn MenuRadioGroup(
    #[prop(optional_no_strip, into)] value: Option<Signal<Option<String>>>,
    #[prop(optional_no_strip, into)] default_value: Option<String>,
    #[prop(optional_no_strip, into)] on_value_change: Option<Callback<(String, &'static str)>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let on_change = on_value_change.map(|on_value_change| {
        Callback::new(move |(value, reason): (Option<String>, &'static str)| {
            if let Some(value) = value {
                on_value_change.run((value, reason));
            }
        })
    });
    let radio = RadioContext {
        value: Controlled::new(value, default_value, on_change),
        disabled,
    };
    let label = RwSignal::new(None::<ElementId<html::Div>>);
    view! {
        <div
            {..attrs}
            role="group"
            aria-labelledby=move || label.get().map(|label| label.get())
            aria-disabled=move || disabled.get().then_some("true")
        >
            <Provider value=GroupLabel(label)>
                <Provider value=radio>{children.map(|children| children())}</Provider>
            </Provider>
        </div>
    }
}

#[component(element = "div", skip(role, tabindex))]
pub fn MenuRadioItem(
    #[prop(into)] value: String,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] close_on_click: Signal<bool>,
    #[prop(optional_no_strip, into)] label: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<MenuContext>();
    let radio = expect_context::<RadioContext>();
    let element = NodeRef::new();
    let disabled =
        Signal::derive(move || disabled.get() || radio.disabled.get() || context.disabled.get());
    let id = register(context, element, label, None);
    let mine = value.clone();
    let checked = Signal::derive(move || radio.value.get().as_deref() == Some(mine.as_str()));
    let value = StoredValue::new(value);
    let select = move || radio.value.set(Some(value.get_value()), reason::ITEM_PRESS);
    let item = Checked {
        checked,
        disabled,
        highlighted: Signal::derive(move || context.highlighted.get() == Some(id)),
    };
    view! {
        <div
            {..attrs}
            {..item_attributes(context, id, disabled, || false)}
            {..press_attributes(context, element, disabled, close_on_click, select)}
            node_ref=element
            role="menuitemradio"
            aria-checked=move || checked.get().to_string()
            data-checked=move || checked.get().then_some("")
            data-unchecked=move || (!checked.get()).then_some("")
        >
            <Provider value=item>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "span")]
pub fn MenuRadioItemIndicator(
    #[prop(optional)] keep_mounted: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let item = expect_context::<Checked>();
    indicator(item, keep_mounted, children)
}

// the group's label, which names it
#[derive(Clone, Copy)]
struct GroupLabel(RwSignal<Option<ElementId<html::Div>>>);

#[component(element = "div", skip(role))]
pub fn MenuGroup(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let label = RwSignal::new(None::<ElementId<html::Div>>);
    view! {
        <div {..attrs} role="group" aria-labelledby=move || label.get().map(|label| label.get())>
            <Provider value=GroupLabel(label)>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "div", skip(id))]
pub fn MenuGroupLabel(
    #[prop(optional_no_strip, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let element = NodeRef::new();
    let id = ElementId::new(element, id.unwrap_or_else(use_id));
    if let Some(GroupLabel(label)) = use_context::<GroupLabel>() {
        label.set(Some(id));
        on_cleanup(move || {
            if label.try_get_untracked().flatten() == Some(id) {
                label.try_set(None);
            }
        });
    }
    view! {
        <div {..attrs} node_ref=element id=id.get() aria-hidden="true">
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div", skip(role, tabindex, id))]
pub fn MenuSubmenuTrigger(
    #[prop(optional_no_strip, into)] id: Option<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(into, default = true.into())] open_on_hover: Signal<bool>,
    #[prop(into, default = SUBMENU_DELAY.into())] delay: Signal<u32>,
    #[prop(optional, into)] close_delay: Signal<u32>,
    #[prop(optional_no_strip, into)] label: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let submenu = expect_context::<MenuContext>();
    let Some(parent) = submenu.parent() else {
        panic!("MenuSubmenuTrigger must be inside a MenuSubmenuRoot");
    };
    if let Some(id) = id {
        submenu.trigger_fallback.set_value(id);
    }
    let element = submenu.row;
    let disabled = Signal::derive(move || disabled.get() || submenu.disabled.get());
    let id = register(parent, element, label, Some(submenu.key));
    let open = move || submenu.open();
    let highlighted = move || parent.highlighted.get() == Some(id);
    let opener = move |reason: &'static str, focus: Focus| {
        if submenu.native.state().get_untracked() {
            if focus == Focus::First {
                submenu.highlight_end(false);
            }
        } else {
            submenu.show(reason, focus);
        }
    };
    view! {
        <div
            {..attrs}
            {..item_attributes(parent, id, disabled, open)}
            node_ref=element
            id=submenu.trigger_fallback.get_value()
            role="menuitem"
            style:anchor-name=submenu.anchor()
            aria-haspopup="menu"
            aria-expanded=move || open().to_string()
            aria-controls=move || open().then(|| submenu.popup_id())
            data-popup-open=move || open().then_some("")
            on:keydown=move |event: ev::KeyboardEvent| {
                let key = event.key();
                let rtl = submenu.direction.get_untracked() == "rtl";
                let into = if rtl { "ArrowLeft" } else { "ArrowRight" };
                if disabled.get_untracked() || (key != into && key != "Enter" && key != " ") {
                    return;
                }
                if key == " " && parent.typing() {
                    return;
                }
                stop(&event);
                opener(reason::LIST_NAVIGATION, Focus::First);
            }
            on:mousedown=move |event: ev::MouseEvent| {
                if event.button() != 0 || disabled.get_untracked() {
                    return;
                }
                if open_on_hover.get_untracked() || !submenu.native.state().get_untracked() {
                    opener(reason::TRIGGER_PRESS, Focus::Nothing);
                } else {
                    submenu.hide(reason::TRIGGER_PRESS);
                }
            }
            // the browser's light dismiss closes a submenu whose trigger is pressed, so it opens again
            on:click=move |event: ev::MouseEvent| {
                if event.detail() != 0 && open_on_hover.get_untracked() && !disabled.get_untracked() {
                    opener(reason::TRIGGER_PRESS, Focus::Nothing);
                }
            }
            on:pointerenter=move |event: ev::PointerEvent| {
                if event.pointer_type() == "mouse"
                    && open_on_hover.get_untracked()
                    && !disabled.get_untracked()
                {
                    submenu.close_delay.set_value(close_delay.get_untracked());
                    let open = Callback::new(move |_| submenu.show(reason::TRIGGER_HOVER, Focus::Nothing));
                    submenu.hover.enter(delay.get_untracked(), open);
                }
            }
            on:pointerleave=move |event: ev::PointerEvent| {
                if event.pointer_type() == "mouse" && open_on_hover.get_untracked() {
                    leave(submenu, &event);
                }
            }
            on:blur=move |_| {
                if highlighted() {
                    parent.highlighted.set(None);
                }
            }
        >
            {children.map(|children| children())}
        </div>
    }
}

// a menu on the press of the right button or a long touch, where the pointer is
#[leptos::component]
pub fn ContextMenuRoot(
    #[prop(optional, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, &'static str)>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(into, default = true.into())] loop_focus: Signal<bool>,
    #[prop(into, default = "vertical".into())] orientation: Signal<&'static str>,
    #[prop(into, default = true.into())] highlight_item_on_hover: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let state = Controlled::new(open, default_open, on_open_change);
    let options = Options {
        disabled,
        modal: true.into(),
        orientation,
        loop_focus,
        highlight_on_hover: highlight_item_on_hover,
        close_parent_on_esc: false.into(),
        context_menu: true,
        bar: None,
    };
    let context = MenuContext::new(state, None, options);
    dismiss(
        move || state.get(),
        move |node| context.contains(node),
        move |reason| {
            // the press that opened it may still be ending, as base ui's
            let fresh = now() - context.opened.get_value().1 < PATIENT_CLICK;
            if reason != reason::OUTSIDE_PRESS || !fresh {
                context.hide(reason);
            }
        },
    );
    view! { <Provider value=context>{children()}</Provider> }
}

#[component(element = "div")]
pub fn ContextMenuTrigger(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<MenuContext>();
    let Some(point) = context.point else {
        panic!("ContextMenuTrigger must be inside a ContextMenuRoot");
    };
    let open = move || context.open();
    let at = move |x: f64, y: f64, size: f64| {
        point.set((x, y, size));
        context.show(reason::TRIGGER_PRESS, Focus::Popup);
    };
    // a touch held still this long opens it, as base ui's
    let press = StoredValue::new(None::<(TimeoutHandle, f64, f64)>);
    let cancel = move || {
        if let Some(Some((handle, _, _))) = press.try_update_value(Option::take) {
            handle.clear();
        }
    };
    view! {
        <div
            {..attrs}
            style:-webkit-touch-callout="none"
            data-popup-open=move || open().then_some("")
            data-pressed=move || open().then_some("")
            on:contextmenu=move |event: ev::MouseEvent| {
                if context.disabled.get_untracked() {
                    return;
                }
                event.prevent_default();
                event.stop_propagation();
                at(f64::from(event.client_x()), f64::from(event.client_y()), 0.0);
            }
            on:pointerdown=move |event: ev::PointerEvent| {
                cancel();
                if event.pointer_type() != "touch" || !event.is_primary() || context.disabled.get_untracked() {
                    return;
                }
                let (x, y) = (f64::from(event.client_x()), f64::from(event.client_y()));
                let handle = set_timeout_with_handle(
                    move || {
                        press.try_set_value(None);
                        at(x, y, 10.0);
                    },
                    Duration::from_millis(LONG_PRESS.into()),
                );
                if let Ok(handle) = handle {
                    press.set_value(Some((handle, x, y)));
                }
            }
            on:pointermove=move |event: ev::PointerEvent| {
                let moved = press.with_value(|press| {
                    press.is_some_and(|(_, x, y)| {
                        (f64::from(event.client_x()) - x).abs() > 10.0
                            || (f64::from(event.client_y()) - y).abs() > 10.0
                    })
                });
                if moved {
                    cancel();
                }
            }
            on:pointerup=move |_| cancel()
            on:pointercancel=move |_| cancel()
        >
            {children.map(|children| children())}
        </div>
    }
}

// one tab stop over its triggers; once a menu is open, the others open as the pointer or the focus reaches them
#[derive(Clone, Copy)]
pub(crate) struct Bar {
    composite: Composite,
    element: NodeRef<html::Div>,
    menus: StoredValue<Vec<MenuContext>>,
    open: StoredValue<Option<MenuContext>>,
    has_open: RwSignal<bool>,
    modal: Signal<bool>,
    disabled: Signal<bool>,
    orientation: Signal<&'static str>,
    loop_focus: Signal<bool>,
    // set while the focus moves to another trigger, so the menu it leaves doesn't take it for a focus out
    switching: StoredValue<bool>,
}

impl Bar {
    fn switch_to(self, menu: MenuContext, reason: &'static str) {
        self.switching.set_value(true);
        if let Some(trigger) = menu.trigger() {
            focus(&trigger);
        }
        menu.show(reason, Focus::Popup);
        self.switching.set_value(false);
    }

    fn step(self, from: MenuContext, forward: bool) {
        let mut menus: Vec<(MenuContext, HtmlElement)> = self
            .menus
            .try_with_value(|menus| {
                menus
                    .iter()
                    .filter_map(|menu| Some((*menu, menu.trigger()?)))
                    .filter(|(_, trigger)| !trigger.has_attribute("disabled"))
                    .collect()
            })
            .unwrap_or_default();
        menus.sort_by(|(_, a), (_, b)| {
            if a.compare_document_position(b) & Node::DOCUMENT_POSITION_FOLLOWING != 0 {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });
        let Some(at) = menus.iter().position(|(menu, _)| menu.key == from.key) else {
            return;
        };
        let looping = self.loop_focus.get_untracked();
        let next = match (forward, at) {
            (true, at) if at + 1 < menus.len() => at + 1,
            (true, _) if looping => 0,
            (false, at) if at > 0 => at - 1,
            (false, _) if looping => menus.len() - 1,
            _ => return,
        };
        if next != at {
            self.switch_to(menus[next].0, reason::TRIGGER_FOCUS);
        }
    }
}

#[component(element = "div", skip(role, id))]
pub fn Menubar(
    #[prop(into, default = true.into())] modal: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(into, default = "horizontal".into())] orientation: Signal<&'static str>,
    #[prop(into, default = true.into())] loop_focus: Signal<bool>,
    #[prop(optional_no_strip, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let element = NodeRef::new();
    let bar = Bar {
        composite: Composite::new(orientation, loop_focus).home_end(),
        element,
        menus: StoredValue::new(Vec::new()),
        open: StoredValue::new(None),
        has_open: RwSignal::new(false),
        modal,
        disabled,
        orientation,
        loop_focus,
        switching: StoredValue::new(false),
    };
    let id = ElementId::new(element, id.unwrap_or_else(use_id));
    view! {
        <div
            {..attrs}
            {..bar.composite.root_attributes()}
            node_ref=element
            role="menubar"
            id=id.get()
            aria-orientation=orientation
            data-orientation=orientation
            data-modal=move || modal.get().then_some("")
            data-has-submenu-open=move || bar.has_open.get().then_some("")
        >
            <Provider value=bar>{children.map(|children| children())}</Provider>
        </div>
    }
}
