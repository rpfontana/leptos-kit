use crate::anchor::{Placement, placement};
use crate::button::button_attributes;
use crate::id::ElementId;
use crate::native::{document_order, keep_gutter, release_gutter};
use crate::typeahead::{Typeahead, Typed};
use crate::{
    Align, ChangeReason, Controlled, Direction, Native, NativeOpen, Orientation, Side, keyword,
    use_direction, use_id,
};
use leptos::context::Provider;
use leptos::either::Either;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::{JsCast, JsValue};
use leptos_kit_macros::component;
use std::time::Duration;
use web_sys::{
    DomRect, FocusOptions, HtmlElement, Node, ScrollIntoViewOptions, ScrollLogicalPosition,
};

const TYPEAHEAD_RESET: u32 = 750;
// a release this long after the press that opened it picks the item under the pointer
const RELEASE_DELAY: u32 = 400;
// or one dragged this far
const DRAG_DISTANCE: f64 = 8.0;
// base ui's room around a popup over its trigger
const MARGIN: f64 = 10.0;
const PADDING: f64 = 5.0;
const MIN_HEIGHT: f64 = 100.0;
// a trigger this close to the viewport's edge gets the popup beside it instead
const TRIGGER_COLLISION: f64 = 20.0;
const EDGE: f64 = 1.0;
const ARROW_STEP: u32 = 40;
// measured unseen at the top left; not by visibility nor seeded size vars, shadcn's popup would transition them
const OVER_TRIGGER: &str = "position:fixed;inset:auto;top:0;left:0;margin:0;padding:0;border:0;\
    background:transparent;color:inherit;overflow:visible;opacity:0;pointer-events:none";

#[derive(Clone, Copy, PartialEq)]
enum Opening {
    Mouse,
    Touch,
    First,
    Last,
}

// the edge a popup over its trigger is pinned to, scrolling grows it from the other
#[derive(Clone, Copy, PartialEq)]
enum Pin {
    Top,
    Bottom,
}

#[derive(Clone, Copy, Default)]
struct Fit {
    placed: bool,
    pin: Option<Pin>,
    full: bool,
}

// a release on an item picks it once the press that opened the popup was held or dragged
#[derive(Clone, Copy, Default)]
struct Release {
    selected: bool,
    unselected: bool,
    drag: f64,
}

#[derive(Clone, Copy)]
struct Entry {
    id: usize,
    element: NodeRef<html::Div>,
    text: NodeRef<html::Div>,
    value: MaybeProp<String>,
    label: Option<Signal<String>>,
    disabled: Signal<bool>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct SelectContext(StoredValue<Select>);

#[derive(Clone, Copy)]
struct Select {
    base: StoredValue<String>,
    positioner_id: ElementId<html::Div>,
    list_id: ElementId<html::Div>,
    popup_id: ElementId<html::Div>,
    native: NativeOpen<html::Div>,
    value: Controlled<Vec<String>>,
    items: Signal<Vec<(Option<String>, String)>>,
    multiple: Signal<bool>,
    disabled: Signal<bool>,
    read_only: Signal<bool>,
    required: Signal<bool>,
    modal: Signal<bool>,
    highlight_on_hover: Signal<bool>,
    direction: Signal<Direction>,
    align_item: StoredValue<Signal<bool>>,
    trigger: NodeRef<html::Button>,
    shown: NodeRef<html::Span>,
    positioner: NodeRef<html::Div>,
    popup: NodeRef<html::Div>,
    list: NodeRef<html::Div>,
    backdrop: NodeRef<html::Div>,
    has_list: RwSignal<bool>,
    has_arrows: RwSignal<bool>,
    entries: StoredValue<Vec<Entry>>,
    next: StoredValue<usize>,
    highlighted: RwSignal<Option<usize>>,
    typeahead: Typeahead,
    opening: RwSignal<Opening>,
    // base ui's pointer modality: a keyboard user's highlight stays when the pointer leaves an item
    pointing: StoredValue<bool>,
    // base ui's alignItemWithTrigger for this opening: the selected item over the trigger's value
    aligned: RwSignal<bool>,
    fit: StoredValue<Fit>,
    release: StoredValue<Release>,
    held: StoredValue<Option<TimeoutHandle>>,
    up: RwSignal<bool>,
    down: RwSignal<bool>,
}

impl Select {
    fn chosen(&self, item: Option<&str>) -> bool {
        let multiple = self.multiple.get();
        self.value.with(|values| chosen(values, multiple, item))
    }

    fn chosen_untracked(&self, item: Option<&str>) -> bool {
        let multiple = self.multiple.get_untracked();
        self.value
            .with_untracked(|values| chosen(values, multiple, item))
    }

    fn has_value(&self) -> bool {
        let multiple = self.multiple.get();
        self.value.with(|values| has_value(values, multiple))
    }

    fn has_value_untracked(&self) -> bool {
        let multiple = self.multiple.get_untracked();
        self.value
            .with_untracked(|values| has_value(values, multiple))
    }
}

fn chosen(values: &[String], multiple: bool, item: Option<&str>) -> bool {
    if multiple {
        item.is_some_and(|item| values.iter().any(|value| value == item))
    } else {
        values.first().map(String::as_str) == item
    }
}

fn has_value(values: &[String], multiple: bool) -> bool {
    if multiple {
        !values.is_empty()
    } else {
        values.first().is_some_and(|value| !value.is_empty())
    }
}

fn label_of(items: &[(Option<String>, String)], value: Option<&str>) -> String {
    items
        .iter()
        .find(|(item, _)| item.as_deref() == value)
        .map_or_else(
            || value.unwrap_or_default().to_string(),
            |(_, label)| label.clone(),
        )
}

impl SelectContext {
    fn select(self) -> Select {
        self.0.get_value()
    }

    fn try_select(self) -> Option<Select> {
        self.0.try_get_value()
    }

    fn open(self) -> bool {
        self.try_select()
            .is_some_and(|select| select.native.state().get())
    }

    fn anchor(self) -> String {
        format!("--{}", self.positioner_id())
    }

    fn positioner_id(self) -> String {
        self.try_select()
            .map(|select| select.positioner_id.get())
            .unwrap_or_default()
    }

    fn listbox_id(self) -> String {
        let Some(select) = self.try_select() else {
            return String::new();
        };
        if select.has_list.get_untracked() {
            select.list_id.get()
        } else {
            select.popup_id.get()
        }
    }

    fn trigger(self) -> Option<HtmlElement> {
        self.try_select()?
            .trigger
            .try_get_untracked()
            .flatten()
            .map(HtmlElement::from)
    }

    fn scroller(self) -> Option<HtmlElement> {
        let select = self.try_select()?;
        let list = select.list.try_get_untracked().flatten();
        match list {
            Some(list) => Some(list.into()),
            None => select
                .popup
                .try_get_untracked()
                .flatten()
                .map(HtmlElement::from),
        }
    }

    fn contains(self, node: &Node) -> bool {
        let Some(select) = self.try_select() else {
            return false;
        };
        let positioner = select
            .positioner
            .try_get_untracked()
            .flatten()
            .map(Node::from);
        [self.trigger().map(Node::from), positioner]
            .iter()
            .flatten()
            .any(|part| part.contains(Some(node)))
    }

    fn register(self, mut entry: Entry) -> usize {
        let select = self.select();
        let id = select
            .next
            .try_update_value(|next| {
                *next += 1;
                *next - 1
            })
            .unwrap_or_default();
        entry.id = id;
        select.entries.update_value(|entries| entries.push(entry));
        on_cleanup(move || {
            select
                .entries
                .try_update_value(|entries| entries.retain(|entry| entry.id != id));
        });
        id
    }

    fn ordered(self) -> Vec<(Entry, HtmlElement)> {
        let mut items: Vec<(Entry, HtmlElement)> = self
            .try_select()
            .and_then(|select| {
                select.entries.try_with_value(|entries| {
                    entries
                        .iter()
                        .filter_map(|entry| {
                            let element = entry.element.try_get_untracked().flatten()?;
                            Some((*entry, element.into()))
                        })
                        .collect()
                })
            })
            .unwrap_or_default();
        items.sort_by(|(_, a), (_, b)| document_order(a, b));
        items
    }

    fn selected_index(self, items: &[(Entry, HtmlElement)]) -> Option<usize> {
        let select = self.try_select()?;
        items
            .iter()
            .position(|(entry, _)| select.chosen_untracked(entry.value.get_untracked().as_deref()))
    }

    fn show(self, reason: ChangeReason, opening: Opening) {
        let Some(select) = self.try_select() else {
            return;
        };
        if select.disabled.get_untracked() || select.native.state().get_untracked() {
            return;
        }
        select.opening.set(opening);
        select
            .aligned
            .set(select.align_item.get_value().get_untracked() && opening != Opening::Touch);
        select
            .pointing
            .set_value(matches!(opening, Opening::Mouse | Opening::Touch));
        select.typeahead.clear();
        select.release.set_value(Release::default());
        let timer = set_timeout_with_handle(
            move || {
                select.release.try_update_value(|release| {
                    release.selected = true;
                    release.unselected = true;
                });
            },
            Duration::from_millis(RELEASE_DELAY.into()),
        );
        let previous = select
            .held
            .try_update_value(|held| std::mem::replace(held, timer.ok()));
        if let Some(Some(handle)) = previous {
            handle.clear();
        }
        select.native.state().set(true, reason);
        // a popup over its trigger stops the page scrolling under it even when it isn't modal
        let locks = select.modal.get_untracked() || select.aligned.get_untracked();
        if locks && select.native.state().get_untracked() {
            self.lock();
        }
    }

    fn hide(self, reason: ChangeReason) {
        let Some(select) = self.try_select() else {
            return;
        };
        if !select.native.state().get_untracked() {
            return;
        }
        select.native.state().set(false, reason);
        self.closed();
        // the focus is on its way somewhere else
        if reason != ChangeReason::FocusOut {
            self.return_focus();
        }
    }

    // once it's closed, whether it was asked to or the browser did it
    fn closed(self) {
        let Some(select) = self.try_select() else {
            return;
        };
        select.highlighted.try_set(None);
        select.up.try_set(false);
        select.down.try_set(false);
        select.fit.try_set_value(Fit::default());
        select.typeahead.clear();
        select.release.try_set_value(Release::default());
        if let Some(Some(handle)) = select.held.try_update_value(Option::take) {
            handle.clear();
        }
        self.unlock();
    }

    // the focus goes back to the trigger unless it already moved somewhere else
    fn return_focus(self) {
        let Some(select) = self.try_select() else {
            return;
        };
        let (Some(positioner), Some(trigger)) = (
            select.positioner.try_get_untracked().flatten(),
            self.trigger(),
        ) else {
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

    // base ui's modal popup: the page can't scroll, a press outside only closes it
    fn lock(self) {
        let Some(select) = self.try_select() else {
            return;
        };
        let Some(backdrop) = select.backdrop.try_get_untracked().flatten() else {
            return;
        };
        keep_gutter();
        let style = HtmlElement::style(&backdrop);
        // the trigger stays reachable through it, as base ui's cutout
        if let Some(trigger) = self.trigger() {
            let r = trigger.get_bounding_client_rect();
            let (left, top, right, bottom) = (r.left(), r.top(), r.right(), r.bottom());
            let clip = format!(
                "polygon(0% 0%,100% 0%,100% 100%,0% 100%,0% 0%,{left}px {top}px,{left}px {bottom}px,\
                 {right}px {bottom}px,{right}px {top}px,{left}px {top}px)"
            );
            let _ = style.set_property("clip-path", &clip);
        }
        let through = if select.modal.get_untracked() {
            "auto"
        } else {
            "none"
        };
        let _ = style.set_property("pointer-events", through);
        let _ = backdrop.show_popover();
    }

    fn unlock(self) {
        let Some(backdrop) = self
            .try_select()
            .and_then(|select| select.backdrop.try_get_untracked().flatten())
        else {
            return;
        };
        if backdrop.matches(":popover-open").unwrap_or(false) {
            let _ = backdrop.hide_popover();
            release_gutter(Some(backdrop.as_ref()));
        }
    }

    // the highlight is the focus, as base ui's list navigation
    fn highlight(self, id: Option<usize>, reveal: bool) {
        let Some(select) = self.try_select() else {
            return;
        };
        select.highlighted.set(id);
        let item = id.and_then(|id| {
            self.ordered()
                .into_iter()
                .find(|(entry, _)| entry.id == id)
                .map(|(_, element)| element)
        });
        if let Some(item) = item {
            focus(&item);
            if reveal {
                scroll_into_view(&item);
            }
        } else if let Some(popup) = select.popup.get_untracked() {
            focus(&popup.into());
        }
    }

    fn take_focus(self) {
        let Some(select) = self.try_select() else {
            return;
        };
        if !select.native.state().get_untracked() {
            return;
        }
        let items = self.ordered();
        let selected = self.selected_index(&items);
        let index = match (selected, select.opening.get_untracked()) {
            (Some(index), _) => Some(index),
            (None, Opening::First) => (!items.is_empty()).then_some(0),
            (None, Opening::Last) => items.len().checked_sub(1),
            (None, _)
                if select.aligned.get_untracked() && select.highlight_on_hover.get_untracked() =>
            {
                (!items.is_empty()).then_some(0)
            }
            (None, _) => None,
        };
        self.highlight(index.map(|index| items[index].0.id), false);
        let Some(item) = selected.map(|index| items[index].1.clone()) else {
            return;
        };
        // an anchored popup takes its size vars with the next render
        if select.aligned.get_untracked() {
            scroll_into_view(&item);
        } else {
            request_animation_frame(move || scroll_into_view(&item));
        }
    }

    fn hovered(self, id: usize, event: &ev::MouseEvent) {
        let Some(select) = self.try_select() else {
            return;
        };
        // webkit sends a move without moving when the page under a still pointer changes
        let still = event.movement_x() == 0 && event.movement_y() == 0;
        let webkit = window()
            .navigator()
            .user_agent()
            .is_ok_and(|agent| agent.contains("AppleWebKit") && !agent.contains("Chrome"));
        if (still && webkit) || !select.native.state().get_untracked() {
            return;
        }
        if select.highlight_on_hover.get_untracked()
            && select.highlighted.get_untracked() != Some(id)
        {
            self.highlight(Some(id), false);
        }
    }

    fn unhovered(self, event: &ev::PointerEvent) {
        let Some(select) = self.try_select() else {
            return;
        };
        if !select.native.state().get_untracked()
            || !select.pointing.get_value()
            || event.pointer_type() == "touch"
            || !select.highlight_on_hover.get_untracked()
        {
            return;
        }
        let items = self.ordered();
        let to = event
            .related_target()
            .and_then(|target| target.dyn_into::<Node>().ok());
        if to.is_some_and(|to| {
            items
                .iter()
                .any(|(_, element)| element.is_same_node(Some(&to)))
        }) {
            return;
        }
        select.highlighted.set(None);
        let Some(popup) = select.popup.get_untracked() else {
            return;
        };
        let inside = document()
            .active_element()
            .is_some_and(|active| popup.contains(Some(&active)));
        if inside {
            focus(&popup.into());
        }
    }

    fn keydown(self, event: &ev::KeyboardEvent) {
        let Some(select) = self.try_select() else {
            return;
        };
        select.pointing.set_value(false);
        let key = event.key();
        match key.as_str() {
            "Escape" => {
                stop(event);
                self.hide(ChangeReason::EscapeKey);
            }
            "Tab" if event.shift_key() => {
                stop(event);
                self.hide(ChangeReason::FocusOut);
                if let Some(trigger) = self.trigger() {
                    focus(&trigger);
                }
            }
            "ArrowDown" | "ArrowUp" | "Home" | "End" => {
                stop(event);
                self.navigate(&key);
            }
            _ => match self.type_ahead(event) {
                Typed::Ignored => {}
                Typed::Missed => stop(event),
                Typed::Matched(entry) => {
                    stop(event);
                    self.highlight(Some(entry.id), true);
                }
            },
        }
    }

    // no looping, and a disabled item takes the highlight too, as base ui's select
    fn navigate(self, key: &str) {
        let Some(select) = self.try_select() else {
            return;
        };
        let items = self.ordered();
        let Some(last) = items.len().checked_sub(1) else {
            return;
        };
        let at = select
            .highlighted
            .get_untracked()
            .and_then(|id| items.iter().position(|(entry, _)| entry.id == id));
        let index = match (key, at) {
            ("Home", _) | ("ArrowDown", None) => 0,
            ("End", _) | ("ArrowUp", None) => last,
            ("ArrowDown", Some(at)) => (at + 1).min(last),
            (_, Some(at)) => at.saturating_sub(1),
            _ => return,
        };
        self.highlight(Some(items[index].0.id), true);
    }

    fn type_ahead(self, event: &ev::KeyboardEvent) -> Typed<Entry> {
        let Some(select) = self.try_select() else {
            return Typed::Ignored;
        };
        let items = self.ordered();
        let labels: Vec<Option<String>> = items
            .iter()
            .map(|(entry, element)| {
                if entry.disabled.get_untracked() {
                    return None;
                }
                let text = entry.text.get_untracked().map(HtmlElement::from);
                entry
                    .label
                    .map(|label| label.get_untracked())
                    .or_else(|| text.and_then(|text| text.text_content()))
                    .or_else(|| element.text_content())
            })
            .collect();
        let at = self.selected_index(&items).or_else(|| {
            select
                .highlighted
                .get_untracked()
                .and_then(|id| items.iter().position(|(entry, _)| entry.id == id))
        });
        select
            .typeahead
            .type_key(event, &labels, at)
            .map(|index| items[index].0)
    }

    fn commit(self, item: Option<String>) {
        let Some(select) = self.try_select() else {
            return;
        };
        if select.disabled.get_untracked() || select.read_only.get_untracked() {
            return;
        }
        if select.multiple.get_untracked() {
            let Some(item) = item else {
                return;
            };
            let mut next = select.value.get_untracked();
            if let Some(at) = next.iter().position(|value| *value == item) {
                next.remove(at);
            } else {
                next.push(item);
            }
            select.value.set(next, ChangeReason::ItemPress);
            return;
        }
        select
            .value
            .set(item.into_iter().collect(), ChangeReason::ItemPress);
        self.hide(ChangeReason::ItemPress);
    }

    fn place(self, placed: Placement, side: Side, side_offset: f64) {
        let Some(select) = self.try_select() else {
            return;
        };
        if !select.native.state().get_untracked() {
            return;
        }
        if select.aligned.get_untracked() {
            if self.align(placed, side, side_offset) {
                self.take_focus();
                return;
            }
            select.aligned.set(false);
        }
        // after the anchored style this queues behind
        queue_microtask(move || {
            let (Some(trigger), Some(Some(positioner)), Some(Some(popup))) = (
                self.trigger(),
                select.positioner.try_get_untracked(),
                select.popup.try_get_untracked(),
            ) else {
                return;
            };
            let _ = HtmlElement::style(&popup).remove_property("--transform-origin");
            placed.settle(&trigger, &positioner);
            self.update_arrows();
            self.take_focus();
        });
    }

    // base ui's alignItemWithTrigger: the selected item's text right over the trigger's value
    fn align(self, placed: Placement, side: Side, side_offset: f64) -> bool {
        let Some(select) = self.try_select() else {
            return false;
        };
        let (Some(trigger), Some(Some(positioner)), Some(Some(popup)), Some(scroller), Some(html)) = (
            self.trigger(),
            select.positioner.try_get_untracked(),
            select.popup.try_get_untracked(),
            self.scroller(),
            document().document_element(),
        ) else {
            return false;
        };
        let (positioner, popup): (HtmlElement, HtmlElement) = (positioner.into(), popup.into());
        // reopened before the last close had tidied up, so the old placing goes first
        let _ = positioner.set_attribute("style", OVER_TRIGGER);
        let (width, height) = (
            f64::from(html.client_width()),
            f64::from(html.client_height()),
        );
        let anchor = trigger.get_bounding_client_rect();
        let style = HtmlElement::style(&positioner);
        let set = |name: &str, value: &str| {
            let _ = style.set_property(name, value);
        };
        let natural = positioner.get_bounding_client_rect().height();
        let (across, along) = hanging(side, side_offset, &anchor, natural, width, height);
        set("--anchor-width", &format!("{}px", anchor.width()));
        set("--anchor-height", &format!("{}px", anchor.height()));
        set("--available-width", &format!("{across}px"));
        set("--available-height", &format!("{along}px"));

        let scale = scale(&trigger);
        let (t, p) = (Rect::of(&trigger, scale), Rect::of(&positioner, scale));
        let computed = window().get_computed_style(&popup).ok().flatten();
        let px = |name: &str| {
            computed
                .as_ref()
                .and_then(|computed| computed.get_property_value(name).ok())
                .and_then(|value| value.strip_suffix("px")?.parse::<f64>().ok())
        };
        let border = px("border-bottom-width").unwrap_or(0.0);
        let max_popup = px("max-height").unwrap_or(f64::INFINITY);
        let scroll_height = f64::from(scroller.scroll_height());
        let viewport = height - 2.0 * MARGIN;
        let below = viewport - t.bottom() + t.height;
        let rtl = select.direction.get_untracked() == Direction::Rtl;
        let mut left = if rtl { t.right() - p.width } else { t.left };
        let mut offset = 0.0;
        let mut text_rect = None;
        if let (Some(text), Some(value)) = (self.aligned_text(), select.shown.get_untracked()) {
            let (v, x) = (Rect::of(&value, scale), Rect::of(&text, scale));
            left = p.left
                + if rtl {
                    v.right() - x.right()
                } else {
                    v.left - x.left
                };
            offset = (x.top - p.top + x.height / 2.0) - (v.top - t.top + v.height / 2.0);
            text_rect = Some(x);
        }
        let ideal = below + offset + MARGIN + border;
        let mut fitted = viewport.min(ideal);
        let max_height = viewport - 2.0 * MARGIN;
        let scroll_top = ideal - fitted;
        let left = left.min(width - PADDING - p.width).max(PADDING);
        set("left", &format!("{}px", left.floor()));
        set("height", &format!("{fitted}px"));
        set("max-height", "none");
        set("margin-top", &format!("{MARGIN}px"));
        set("margin-bottom", &format!("{MARGIN}px"));
        let _ = HtmlElement::style(&popup).set_property("height", "100%");

        let most = max_scroll(&scroller);
        let top_pinned = scroll_top >= most - EDGE;
        if top_pinned {
            fitted = viewport.min(p.height) - (scroll_top - most);
        }
        let near_edge = t.top < TRIGGER_COLLISION || t.bottom() > viewport - TRIGGER_COLLISION;
        if near_edge || fitted.ceil() + EDGE < scroll_height.min(MIN_HEIGHT) {
            return false;
        }
        let (pin, rise) = if top_pinned {
            let top = if p.height >= max_height {
                0.0
            } else {
                (viewport - ideal).max(0.0)
            };
            set("top", &format!("{}px", top.floor()));
            set("height", &format!("{fitted}px"));
            set_scroll_top(&scroller, max_scroll(&scroller));
            ((top == 0.0).then_some(Pin::Top), top - top.floor())
        } else {
            // raised by the fraction its top would fall on
            let at = height - MARGIN - fitted;
            let rise = at - at.floor();
            set("top", "auto");
            set("bottom", &format!("{rise}px"));
            set_scroll_top(&scroller, scroll_top);
            (Some(Pin::Bottom), rise)
        };
        // set here with transitions off, shadcn's duration-100 would slide the popup by the fraction
        let (shift, popup_style) = (left - left.floor(), HtmlElement::style(&popup));
        let _ = popup_style.set_property("transition", "none");
        let _ = popup_style.set_property("left", &format!("{shift}px"));
        let _ = popup_style.set_property("top", &format!("{rise}px"));
        if let Ok(Some(computed)) = window().get_computed_style(&popup) {
            let _ = computed.get_property_value("left");
        }
        let _ = popup_style.remove_property("transition");
        placed.shift(shift, rise);
        set("opacity", "1");
        set("pointer-events", "auto");
        if let Some(x) = text_rect {
            let center = x.top + x.height / 2.0;
            let y = if p.height > 0.0 {
                ((center - p.top) / p.height * 100.0).clamp(0.0, 100.0)
            } else {
                50.0
            };
            let _ =
                HtmlElement::style(&popup).set_property("--transform-origin", &format!("50% {y}%"));
        }
        select.fit.set_value(Fit {
            placed: true,
            pin,
            full: MIN_HEIGHT.max(fitted) == viewport || fitted >= max_popup,
        });
        self.update_arrows();
        true
    }

    // the selected item's text, or with nothing selected the first item's
    fn aligned_text(self) -> Option<HtmlElement> {
        let select = self.try_select()?;
        let items = self.ordered();
        let text = |entry: &Entry| entry.text.get_untracked().map(HtmlElement::from);
        if let Some(index) = self.selected_index(&items) {
            return text(&items[index].0);
        }
        if select.has_value_untracked() {
            return None;
        }
        items.first().and_then(|(entry, _)| text(entry))
    }

    // a popup over its trigger pinned to one edge grows towards the other as it scrolls
    fn scrolled(self) {
        let Some(select) = self.try_select() else {
            return;
        };
        let (Some(Some(positioner)), Some(Some(popup)), Some(scroller), Some(html)) = (
            select.positioner.try_get_untracked(),
            select.popup.try_get_untracked(),
            self.scroller(),
            document().document_element(),
        ) else {
            return;
        };
        let fit = select.fit.get_value();
        if !fit.placed {
            return;
        }
        let Some(pin) = fit
            .pin
            .filter(|_| !fit.full && select.aligned.get_untracked())
        else {
            self.update_arrows();
            return;
        };
        let positioner: HtmlElement = positioner.into();
        let current = positioner.get_bounding_client_rect().height();
        let max_popup = window()
            .get_computed_style(&popup)
            .ok()
            .flatten()
            .and_then(|computed| computed.get_property_value("max-height").ok())
            .and_then(|value| value.strip_suffix("px")?.parse::<f64>().ok())
            .unwrap_or(f64::INFINITY);
        let room = (f64::from(html.client_height()) - 2.0 * MARGIN).min(max_popup);
        let top = scroll_top(&scroller);
        let most = max_scroll(&scroller);
        let diff = if pin == Pin::Top { most - top } else { top };
        let next = (current + diff).min(room);
        let style = HtmlElement::style(&positioner);
        let full = if diff <= EDGE {
            let delta = diff.min(room - current).max(0.0);
            if delta > 0.0 {
                let _ = style.set_property("height", &format!("{}px", current + delta));
            }
            set_scroll_top(&scroller, if pin == Pin::Top { most } else { 0.0 });
            room - (current + delta) <= EDGE
        } else {
            let target = if room - next > EDGE {
                Some(if pin == Pin::Top { f64::INFINITY } else { 0.0 })
            } else if pin == Pin::Bottom && top < most {
                let overshoot = current + diff - room;
                Some(top - (diff - overshoot))
            } else {
                None
            };
            let height = next.ceil();
            if height != 0.0 {
                let _ = style.set_property("height", &format!("{height}px"));
            }
            if let Some(target) = target {
                let target = target.min(max_scroll(&scroller)).max(0.0);
                if (scroll_top(&scroller) - target).abs() > EDGE {
                    set_scroll_top(&scroller, target);
                }
            }
            height >= room - EDGE
        };
        if full {
            select.fit.update_value(|fit| fit.full = true);
        }
        self.update_arrows();
    }

    fn update_arrows(self) {
        let (Some(select), Some(scroller)) = (self.try_select(), self.scroller()) else {
            return;
        };
        let most = max_scroll(&scroller);
        let top = normalize(scroll_top(&scroller), most);
        select.up.try_set(top > 0.0);
        select.down.try_set(top < most);
    }

    // one more item into view, as base ui's scroll arrows; false once there's no more
    fn step(self, up: bool, arrow: f64) -> bool {
        let (Some(select), Some(scroller)) = (self.try_select(), self.scroller()) else {
            return false;
        };
        select.highlighted.try_set(None);
        self.update_arrows();
        let most = max_scroll(&scroller);
        let raw = scroll_top(&scroller);
        let top = normalize(raw, most);
        if top != raw {
            set_scroll_top(&scroller, top);
        }
        if top == if up { 0.0 } else { most } {
            return false;
        }
        let items: Vec<HtmlElement> = self
            .ordered()
            .into_iter()
            .map(|(_, element)| element)
            .collect();
        let Some(last) = items.len().checked_sub(1) else {
            return true;
        };
        let start = |item: &HtmlElement| f64::from(item.offset_top());
        let end = |item: &HtmlElement| f64::from(item.offset_top() + item.offset_height());
        let target = if up {
            let first = items
                .iter()
                .position(|item| start(item) >= top + arrow - EDGE)
                .unwrap_or(0);
            match first.checked_sub(1) {
                Some(target) => normalize(start(&items[target]) - arrow, most),
                None => 0.0,
            }
        } else {
            let client = f64::from(scroller.client_height());
            let seen = items
                .iter()
                .position(|item| end(item) > top + client - arrow + EDGE)
                .map_or(last, |index| index.saturating_sub(1));
            if seen < last {
                normalize(end(&items[seen + 1]) - client + arrow, most)
            } else {
                most
            }
        };
        set_scroll_top(&scroller, target);
        true
    }
}

#[derive(Clone, Copy)]
struct Rect {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}

impl Rect {
    // floating-ui's normalizeRect, through the trigger's scale
    fn of(element: &web_sys::Element, (x, y): (f64, f64)) -> Self {
        let r = element.get_bounding_client_rect();
        Self {
            left: r.left() / x,
            top: r.top() / y,
            width: r.width() / x,
            height: r.height() / y,
        }
    }

    fn right(self) -> f64 {
        self.left + self.width
    }

    fn bottom(self) -> f64 {
        self.top + self.height
    }
}

// floating-ui's getScale: rarely 1 exactly, computed style rounds the width to three decimals
fn scale(element: &HtmlElement) -> (f64, f64) {
    let r = element.get_bounding_client_rect();
    let css = window().get_computed_style(element).ok().flatten();
    let px = |name: &str| {
        css.as_ref()
            .and_then(|css| css.get_property_value(name).ok())
            .and_then(|value| value.strip_suffix("px")?.parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    let (mut width, mut height) = (px("width"), px("height"));
    let (offset_width, offset_height) = (
        f64::from(element.offset_width()),
        f64::from(element.offset_height()),
    );
    let fallback = width.round() != offset_width || height.round() != offset_height;
    if fallback {
        (width, height) = (offset_width, offset_height);
    }
    let axis = |rendered: f64, css: f64| {
        let ratio = if fallback { rendered.round() } else { rendered } / css;
        if ratio.is_finite() && ratio != 0.0 {
            ratio
        } else {
            1.0
        }
    };
    (axis(r.width(), width), axis(r.height(), height))
}

// floating-ui's flip and size for the popup hung off the trigger, which its size vars come from
fn hanging(side: Side, offset: f64, t: &DomRect, height: f64, width: f64, tall: f64) -> (f64, f64) {
    let across = width - 2.0 * PADDING;
    let (preferred, other) = match side {
        Side::Top => (Side::Top, Side::Bottom),
        Side::Bottom => (Side::Bottom, Side::Top),
        _ => return (across, tall - 2.0 * PADDING),
    };
    let room = |side: Side| match side {
        Side::Top => t.top() - offset - PADDING,
        _ => tall - t.bottom() - offset - PADDING,
    };
    // flip keeps a pixel more off the edges, and one more off the far one
    let (near, far) = (room(preferred) - 1.0, room(other) - 2.0);
    let shown = if height <= near {
        preferred
    } else if height <= far || far > near {
        other
    } else {
        preferred
    };
    (across, room(shown))
}

fn scroll_top(element: &HtmlElement) -> f64 {
    web_sys::js_sys::Reflect::get(element, &JsValue::from_str("scrollTop"))
        .ok()
        .and_then(|top| top.as_f64())
        .unwrap_or(0.0)
}

// web-sys only takes whole pixels, base ui sets fractions
fn set_scroll_top(element: &HtmlElement, top: f64) {
    let _ = web_sys::js_sys::Reflect::set(
        element,
        &JsValue::from_str("scrollTop"),
        &JsValue::from_f64(top),
    );
}

fn max_scroll(element: &HtmlElement) -> f64 {
    f64::from(element.scroll_height() - element.client_height()).max(0.0)
}

// within a pixel of an end counts as at it
fn normalize(top: f64, most: f64) -> f64 {
    if most <= 0.0 {
        return 0.0;
    }
    let top = top.min(most).max(0.0);
    match (top <= EDGE, most - top <= EDGE) {
        (true, true) if top <= most - top => 0.0,
        (_, true) => most,
        (true, false) => 0.0,
        (false, false) => top,
    }
}

fn focus(element: &HtmlElement) {
    let options = FocusOptions::new();
    options.set_prevent_scroll(true);
    let _ = element.focus_with_options(&options);
}

fn scroll_into_view(element: &HtmlElement) {
    let options = ScrollIntoViewOptions::new();
    options.set_block(ScrollLogicalPosition::Nearest);
    options.set_inline(ScrollLogicalPosition::Nearest);
    element.scroll_into_view_with_scroll_into_view_options(&options);
}

fn stop(event: &ev::KeyboardEvent) {
    event.prevent_default();
    event.stop_propagation();
}

#[leptos::component]
pub fn SelectRoot(
    #[prop(optional, into)] value: Option<Signal<Vec<String>>>,
    #[prop(optional)] default_value: Vec<String>,
    #[prop(optional, into)] on_value_change: Option<Callback<(Vec<String>, ChangeReason)>>,
    #[prop(optional, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, ChangeReason)>>,
    #[prop(optional, into)] items: Signal<Vec<(Option<String>, String)>>,
    #[prop(optional, into)] multiple: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] read_only: Signal<bool>,
    #[prop(optional, into)] required: Signal<bool>,
    #[prop(into, default = true.into())] modal: Signal<bool>,
    #[prop(into, default = true.into())] highlight_item_on_hover: Signal<bool>,
    #[prop(optional, into)] name: Option<Signal<String>>,
    #[prop(optional, into)] form: Option<Signal<String>>,
    #[prop(optional, into)] id: Option<String>,
    children: Children,
) -> impl IntoView {
    let base = id.unwrap_or_else(use_id);
    let (positioner, popup, list) = (NodeRef::new(), NodeRef::new(), NodeRef::new());
    let state = Controlled::new(open, default_open, on_open_change);
    let value = Controlled::new(value, default_value, on_value_change);
    let context = SelectContext(StoredValue::new(Select {
        positioner_id: ElementId::new(positioner, format!("{base}-positioner")),
        list_id: ElementId::new(list, format!("{base}-list")),
        popup_id: ElementId::new(popup, format!("{base}-list")),
        base: StoredValue::new(base),
        native: NativeOpen::new(Native::Popover, positioner, state, Some(popup)),
        value,
        items,
        multiple,
        disabled,
        read_only,
        required,
        modal,
        highlight_on_hover: highlight_item_on_hover,
        direction: use_direction(),
        align_item: StoredValue::new(true.into()),
        trigger: NodeRef::new(),
        shown: NodeRef::new(),
        positioner,
        popup,
        list,
        backdrop: NodeRef::new(),
        has_list: RwSignal::new(false),
        has_arrows: RwSignal::new(false),
        entries: StoredValue::new(Vec::new()),
        next: StoredValue::new(0),
        highlighted: RwSignal::new(None),
        typeahead: Typeahead::new(TYPEAHEAD_RESET),
        opening: RwSignal::new(Opening::Mouse),
        pointing: StoredValue::new(true),
        aligned: RwSignal::new(false),
        fit: StoredValue::new(Fit::default()),
        release: StoredValue::new(Release::default()),
        held: StoredValue::new(None),
        up: RwSignal::new(false),
        down: RwSignal::new(false),
    }));
    on_cleanup(move || {
        context.unlock();
        if let Some(Some(handle)) = context
            .try_select()
            .and_then(|select| select.held.try_update_value(Option::take))
        {
            handle.clear();
        }
    });
    let inputs = name.map(|name| {
        move || {
            let mut values = value.get();
            if !multiple.get() {
                values = vec![values.into_iter().next().unwrap_or_default()];
            }
            values
                .into_iter()
                .map(|value| {
                    view! {
                        <input
                            type="hidden"
                            name=move || name.get()
                            form=move || form.map(|form| form.get())
                            value=value
                            disabled=move || disabled.get()
                        />
                    }
                })
                .collect_view()
        }
    });
    view! {
        <Provider value=context>{children()}</Provider>
        {inputs}
    }
}

#[component(
    element = "button",
    skip(r#type, id, role, popovertarget, popovertargetaction)
)]
pub fn SelectTrigger(
    #[prop(optional_no_strip, into)] disabled: Option<Signal<bool>>,
    #[prop(optional_no_strip, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<SelectContext>();
    let select = context.select();
    let disabled = move || select.disabled.get() || disabled.is_some_and(|disabled| disabled.get());
    let open = move || context.open();
    let tabindex = Signal::derive(move || if disabled() { -1 } else { 0 });
    let touch = StoredValue::new(false);
    let pressing = StoredValue::new(false);
    let release = StoredValue::new(None::<WindowListenerHandle>);
    on_cleanup(move || {
        if let Some(Some(handle)) = release.try_update_value(Option::take) {
            handle.remove();
        }
    });
    // a press released away from the trigger and the popup takes the opening back
    let released = move |event: ev::MouseEvent| {
        if let Some(Some(handle)) = release.try_update_value(Option::take) {
            handle.remove();
        }
        pressing.set_value(false);
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
            context.hide(ChangeReason::CancelOpen);
        }
    };

    view! {
        <button
            {..attrs}
            {..button_attributes(disabled, || false, Some(tabindex))}
            node_ref=select.trigger
            id=id.unwrap_or_else(|| select.base.get_value())
            type="button"
            role="combobox"
            style:anchor-name=context.anchor()
            popovertarget=context.positioner_id()
            aria-haspopup="listbox"
            aria-expanded=move || open().to_string()
            aria-controls=move || open().then(|| context.listbox_id())
            aria-readonly=move || select.read_only.get().then_some("true")
            aria-required=move || select.required.get().then_some("true")
            data-popup-open=move || open().then_some("")
            data-pressed=move || open().then_some("")
            data-placeholder=move || (!select.has_value()).then_some("")
            data-readonly=move || select.read_only.get().then_some("")
            on:pointerdown=move |event: ev::PointerEvent| touch.set_value(event.pointer_type() == "touch")
            on:mousedown=move |event: ev::MouseEvent| {
                if event.button() != 0 || disabled() {
                    return;
                }
                if select.native.state().get_untracked() {
                    context.hide(ChangeReason::TriggerPress);
                    return;
                }
                let opening = if touch.get_value() { Opening::Touch } else { Opening::Mouse };
                // now rather than base ui's next frame, so it's placed before it's first painted
                pressing.set_value(true);
                context.show(ChangeReason::TriggerPress, opening);
                release.set_value(Some(window_event_listener(ev::mouseup, released)));
            }
            on:click=move |event: ev::MouseEvent| {
                // the press already did it, and the browser's own toggle would undo it
                event.prevent_default();
                if event.detail() != 0 || disabled() {
                    return;
                }
                if select.native.state().get_untracked() {
                    context.hide(ChangeReason::TriggerPress);
                } else {
                    context.show(ChangeReason::TriggerPress, Opening::First);
                }
            }
            on:keydown=move |event: ev::KeyboardEvent| {
                if disabled() || select.native.state().get_untracked() {
                    return;
                }
                let key = event.key();
                if key == "ArrowDown" || key == "ArrowUp" {
                    event.prevent_default();
                    let opening = if key == "ArrowUp" { Opening::Last } else { Opening::First };
                    context.show(ChangeReason::ListNavigation, opening);
                    return;
                }
                // typing picks an item straight away while it's closed
                if select.multiple.get_untracked() || select.read_only.get_untracked() {
                    return;
                }
                let typed = context.type_ahead(&event);
                if key == " " && !matches!(typed, Typed::Ignored) {
                    event.prevent_default();
                }
                if let Typed::Matched(entry) = typed {
                    let next = entry.value.get_untracked().into_iter().collect();
                    select.value.set(next, ChangeReason::None);
                }
            }
            // not for the focus the opening press itself gives it
            on:focus=move |_| {
                let open = select.native.state().get_untracked();
                if open && select.aligned.get_untracked() && !pressing.get_value() {
                    context.hide(ChangeReason::None);
                }
            }
        >
            {children.map(|children| children())}
        </button>
    }
}

#[component(element = "span")]
pub fn SelectValue(
    #[prop(optional, into)] placeholder: MaybeProp<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<SelectContext>();
    let select = context.select();
    let shown = move || {
        let null_label = select
            .items
            .with(|items| items.iter().any(|(value, _)| value.is_none()));
        if !select.has_value()
            && !null_label
            && let Some(placeholder) = placeholder.get()
        {
            return placeholder;
        }
        let values = select.value.get();
        select.items.with(|items| {
            if select.multiple.get() {
                values
                    .iter()
                    .map(|value| label_of(items, Some(value)))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                label_of(items, values.first().map(String::as_str))
            }
        })
    };
    let inner = match children {
        Some(children) => Either::Left(children()),
        None => Either::Right(shown),
    };
    view! {
        <span
            {..attrs}
            node_ref=select.shown
            data-placeholder=move || (!select.has_value()).then_some("")
        >
            {inner}
        </span>
    }
}

// base ui's Select.Icon state, for an icon of the caller's own that hides itself
pub fn select_icon_attributes() -> impl leptos::attr::Attribute {
    let context = expect_context::<SelectContext>();
    view! { <{..} data-popup-open=move || context.open().then_some("") /> }
}

#[component(element = "span")]
pub fn SelectIcon(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let content = match children {
        Some(children) => Either::Left(children()),
        None => Either::Right("▼"),
    };
    view! {
        <span {..attrs} {..select_icon_attributes()} aria-hidden="true">
            {content}
        </span>
    }
}

#[leptos::component]
pub fn SelectPortal(children: Children) -> impl IntoView {
    children()
}

#[component(element = "div", skip(id, style, role))]
pub fn SelectPositioner(
    #[prop(optional, into)] side: Signal<Side>,
    #[prop(optional, into)] align: Signal<Align>,
    #[prop(optional, into)] side_offset: Signal<f64>,
    #[prop(optional, into)] align_offset: Signal<f64>,
    #[prop(into, default = true.into())] align_item_with_trigger: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<SelectContext>();
    let select = context.select();
    select.align_item.set_value(align_item_with_trigger);
    let placed = Placement::new(side, align);
    let open = move || context.open();
    let aligned = select.aligned;
    // over its trigger it doesn't read open, a closing popup keeps the place it was given
    let style = move || {
        if aligned.get() {
            OVER_TRIGGER.to_string()
        } else {
            let inert = if open() { "" } else { ";pointer-events:none" };
            let place = placement(
                &context.anchor(),
                side.get(),
                align.get(),
                side_offset.get(),
                align_offset.get(),
            );
            format!("{place};{}{inert}", (placed.sizes())())
        }
    };
    // show_popover runs in the root's effect and the style lands with this render, so a frame later
    Effect::new(move || {
        if select.native.state().get() {
            request_animation_frame(move || {
                context.place(placed, side.get_untracked(), side_offset.get_untracked());
            });
        }
    });
    // base ui closes a popup over its trigger rather than follow the trigger
    let resize = StoredValue::new(None::<WindowListenerHandle>);
    let unlisten = move || {
        if let Some(Some(handle)) = resize.try_update_value(Option::take) {
            handle.remove();
        }
    };
    Effect::new(move || {
        unlisten();
        if select.native.state().get() && aligned.get() {
            let handle = window_event_listener(ev::resize, move |_| {
                context.hide(ChangeReason::WindowResize);
            });
            resize.set_value(Some(handle));
        }
    });
    on_cleanup(unlisten);
    let side_shown = move || {
        if aligned.get() {
            "none"
        } else {
            placed.side().as_str()
        }
    };
    view! {
        <div
            node_ref=select.backdrop
            popover="manual"
            role="presentation"
            data-modal-backdrop=""
            style="position:fixed;inset:0;width:auto;height:auto;max-width:none;max-height:none;margin:0;padding:0;border:0;background:transparent;user-select:none;-webkit-user-select:none"
        ></div>
        <div
            {..attrs}
            node_ref=select.positioner
            id=context.positioner_id()
            popover="auto"
            role="presentation"
            style=style
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-side=side_shown
            data-align=keyword(placed.align)
            // only the browser's own close is news, ours already tidied up and may have reopened since
            on:toggle=move |_| {
                let was = select.native.state().try_get_untracked() == Some(true);
                select.native.sync(ChangeReason::OutsidePress);
                if was && select.native.state().try_get_untracked() == Some(false) {
                    context.closed();
                    context.return_focus();
                }
                // back to the anchored layout once nobody can see it
                let hidden = select
                    .positioner
                    .get_untracked()
                    .is_some_and(|positioner| !positioner.matches(":popover-open").unwrap_or(false));
                if hidden && select.native.state().try_get_untracked() == Some(false) {
                    aligned.set(false);
                    placed.reset();
                }
            }
        >
            <Provider value=placed>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "div", skip(id, role, tabindex))]
pub fn SelectPopup(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<SelectContext>();
    let placed = expect_context::<Placement>();
    let select = context.select();
    let open = move || context.open();
    let aligned = move || select.aligned.get();
    let listbox = move || !select.has_list.get();
    // without a list the popup scrolls itself
    let lone = move || aligned() && listbox();
    view! {
        <div
            {..attrs}
            node_ref=select.popup
            id=move || listbox().then(|| select.popup_id.get())
            role=move || if listbox() { "listbox" } else { "presentation" }
            aria-multiselectable=move || (listbox() && select.multiple.get()).then_some("true")
            aria-readonly=move || (listbox() && select.read_only.get()).then_some("true")
            tabindex="-1"
            style:position="relative"
            style:left=placed.nudge(|(x, _)| x)
            style:top=placed.nudge(|(_, y)| y)
            style:height=move || aligned().then_some("100%")
            style:max-height=move || lone().then_some("100%")
            style:overflow-y=move || lone().then_some("auto")
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-side=move || if aligned() { "none" } else { placed.side().as_str() }
            data-align=keyword(placed.align)
            on:keydown=move |event: ev::KeyboardEvent| context.keydown(&event)
            on:pointermove=move |_| select.pointing.set_value(true)
            on:focusout=move |event: ev::FocusEvent| {
                let to = event.related_target().and_then(|target| target.dyn_into::<Node>().ok());
                if to.is_some_and(|to| !context.contains(&to)) {
                    context.hide(ChangeReason::FocusOut);
                }
            }
            on:scroll=move |_| {
                if !select.has_list.get_untracked() {
                    context.scrolled();
                }
            }
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div", skip(id, role))]
pub fn SelectList(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<SelectContext>();
    let select = context.select();
    select.has_list.set(true);
    on_cleanup(move || {
        select.has_list.try_set(false);
    });
    let aligned = move || select.aligned.get();
    // the arrows scroll it, so its own scrollbar goes
    let bare = move || select.has_arrows.get() && select.opening.get() != Opening::Touch;
    view! {
        <div
            {..attrs}
            node_ref=select.list
            id=select.list_id.get()
            role="listbox"
            aria-multiselectable=move || select.multiple.get().then_some("true")
            aria-readonly=move || select.read_only.get().then_some("true")
            style:position=move || aligned().then_some("relative")
            style:max-height=move || aligned().then_some("100%")
            style:overflow-x=move || aligned().then_some("hidden")
            style:overflow-y=move || aligned().then_some("auto")
            style:scrollbar-width=move || bare().then_some("none")
            on:scroll=move |_| context.scrolled()
        >
            {children.map(|children| children())}
        </div>
    }
}

#[derive(Clone, Copy)]
struct ItemContext {
    text: NodeRef<html::Div>,
    selected: Signal<bool>,
}

#[component(element = "div", skip(role, tabindex))]
pub fn SelectItem(
    #[prop(optional, into)] value: MaybeProp<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional_no_strip, into)] label: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<SelectContext>();
    let select = context.select();
    let element = NodeRef::new();
    let text = NodeRef::new();
    let disabled = Signal::derive(move || disabled.get() || select.disabled.get());
    let id = context.register(Entry {
        id: 0,
        element,
        text,
        value,
        label,
        disabled,
    });
    let selected = Signal::derive(move || select.chosen(value.get().as_deref()));
    let highlighted = move || select.highlighted.get() == Some(id);
    let pointer = StoredValue::new(String::from("mouse"));
    // base ui's allowMouseSelection: the press began on this item
    let pressed = StoredValue::new(false);
    let touch = move || pointer.with_value(|pointer| pointer == "touch");
    let click = move || {
        if let Some(element) = element.get_untracked() {
            element.click();
        }
    };
    view! {
        <div
            {..attrs}
            node_ref=element
            role="option"
            aria-selected=move || selected.get().to_string()
            aria-disabled=move || disabled.get().then_some("true")
            tabindex=move || if context.open() && highlighted() { 0 } else { -1 }
            data-selected=move || selected.get().then_some("")
            data-highlighted=move || highlighted().then_some("")
            data-disabled=move || disabled.get().then_some("")
            on:focus=move |_| select.highlighted.set(Some(id))
            on:mousemove=move |event: ev::MouseEvent| context.hovered(id, &event)
            on:pointerleave=move |event: ev::PointerEvent| context.unhovered(&event)
            on:pointerenter=move |event: ev::PointerEvent| pointer.set_value(event.pointer_type())
            on:pointerdown=move |event: ev::PointerEvent| {
                pointer.set_value(event.pointer_type());
                pressed.set_value(true);
                select.release.update_value(|release| release.drag = 0.0);
            }
            on:pointermove=move |event: ev::PointerEvent| {
                if event.pointer_type() == "mouse" && event.buttons() == 1 {
                    select.release.update_value(|release| {
                        release.drag += f64::from(event.movement_y());
                        if release.drag.abs() >= DRAG_DISTANCE {
                            release.unselected = true;
                        }
                    });
                }
            }
            on:mouseup=move |_| {
                select.release.update_value(|release| release.drag = 0.0);
                if disabled.get_untracked() || touch() || pressed.get_value() {
                    return;
                }
                let release = select.release.get_value();
                let allowed = if selected.get_untracked() { release.selected } else { release.unselected };
                if allowed {
                    pressed.set_value(true);
                    click();
                    pressed.set_value(false);
                }
            }
            on:click=move |event: ev::MouseEvent| {
                // a keyboard's click comes without a press, a pointer's needs one on this item
                let valid = touch() || pressed.get_value() || event.detail() == 0;
                pressed.set_value(false);
                if valid && !disabled.get_untracked() {
                    context.commit(value.get_untracked());
                }
            }
            on:keydown=move |event: ev::KeyboardEvent| {
                let key = event.key();
                if key == " " && select.typeahead.typing() {
                    event.prevent_default();
                    return;
                }
                if (key == "Enter" || key == " ") && !disabled.get_untracked() {
                    event.prevent_default();
                    click();
                }
            }
        >
            <Provider value=ItemContext { text, selected }>
                {children.map(|children| children())}
            </Provider>
        </div>
    }
}

#[component(element = "div")]
pub fn SelectItemText(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let item = expect_context::<ItemContext>();
    view! {
        <div {..attrs} node_ref=item.text>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "span")]
pub fn SelectItemIndicator(
    #[prop(optional)] keep_mounted: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let selected = expect_context::<ItemContext>().selected;
    let content = match children {
        Some(children) => Either::Left(children()),
        None => Either::Right("✔️"),
    };
    view! {
        <span
            {..attrs}
            aria-hidden="true"
            hidden=move || !keep_mounted && !selected.get()
            data-selected=move || selected.get().then_some("")
        >
            {content}
        </span>
    }
}

#[derive(Clone, Copy)]
struct GroupLabel(RwSignal<Option<ElementId<html::Div>>>);

#[component(element = "div", skip(role))]
pub fn SelectGroup(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let label = RwSignal::new(None::<ElementId<html::Div>>);
    view! {
        <div {..attrs} role="group" aria-labelledby=move || label.get().map(|label| label.get())>
            <Provider value=GroupLabel(label)>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "div", skip(id))]
pub fn SelectGroupLabel(
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

#[component(element = "div", skip(role))]
pub fn SelectSeparator(
    #[prop(into, default = Orientation::Horizontal.into())] orientation: Signal<Orientation>,
) -> impl IntoView {
    view! { <div {..attrs} role="presentation" data-orientation=keyword(orientation)></div> }
}

fn arrow_attributes(up: bool, keep_mounted: bool) -> impl leptos::attr::Attribute {
    let context = expect_context::<SelectContext>();
    let placed = expect_context::<Placement>();
    let select = context.select();
    select.has_arrows.set(true);
    let visible = move || {
        let room = if up {
            select.up.get()
        } else {
            select.down.get()
        };
        room && select.opening.get() != Opening::Touch
    };
    let timer = StoredValue::new(None::<TimeoutHandle>);
    let halt = move || {
        if let Some(Some(handle)) = timer.try_update_value(Option::take) {
            handle.clear();
        }
    };
    on_cleanup(halt);
    view! {
        <{..}
            aria-hidden="true"
            hidden=move || !keep_mounted && !visible()
            data-direction=if up { "up" } else { "down" }
            data-side=move || if select.aligned.get() { "none" } else { placed.side().as_str() }
            data-visible=move || visible().then_some("")
            style:position="absolute"
            on:mousemove=move |event: ev::MouseEvent| {
                let still = event.movement_x() == 0 && event.movement_y() == 0;
                if still || timer.with_value(Option::is_some) {
                    return;
                }
                select.highlighted.set(None);
                let height = event
                    .current_target()
                    .and_then(|arrow| arrow.dyn_into::<HtmlElement>().ok())
                    .map_or(0.0, |arrow| f64::from(arrow.offset_height()));
                scroll_on(context, timer, up, height);
            }
            on:mouseleave=move |_| halt()
        />
    }
}

fn scroll_on(
    context: SelectContext,
    timer: StoredValue<Option<TimeoutHandle>>,
    up: bool,
    height: f64,
) {
    let handle = set_timeout_with_handle(
        move || {
            timer.try_set_value(None);
            if context.step(up, height) {
                scroll_on(context, timer, up, height);
            }
        },
        Duration::from_millis(ARROW_STEP.into()),
    );
    timer.try_set_value(handle.ok());
}

#[component(element = "div")]
pub fn SelectScrollUpArrow(
    #[prop(optional)] keep_mounted: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let content = match children {
        Some(children) => Either::Left(children()),
        None => Either::Right("▲"),
    };
    view! { <div {..attrs} {..arrow_attributes(true, keep_mounted)}>{content}</div> }
}

#[component(element = "div")]
pub fn SelectScrollDownArrow(
    #[prop(optional)] keep_mounted: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let content = match children {
        Some(children) => Either::Left(children()),
        None => Either::Right("▼"),
    };
    view! { <div {..attrs} {..arrow_attributes(false, keep_mounted)}>{content}</div> }
}
