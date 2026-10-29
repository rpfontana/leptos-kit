use crate::anchor::{Placement, placement};
use crate::button::button_attributes;
use crate::dismiss::dismiss;
use crate::id::ElementId;
use crate::native::{document_order, observe_resize};
use crate::{
    Align, ChangeReason, Controlled, Direction, Native, NativeOpen, Orientation, Side, keyword,
    use_direction, use_id,
};
use leptos::context::Provider;
use leptos::either::Either;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_kit_macros::component;
use std::fmt;
use web_sys::{FocusOptions, HtmlElement, Node, ScrollIntoViewOptions, ScrollLogicalPosition};

// what the arrows that open it ask to highlight
#[derive(Clone, Copy, PartialEq)]
enum Opening {
    Pointer,
    Typing,
    First,
    Last,
}

#[derive(Clone, Copy)]
struct Entry {
    id: usize,
    element: NodeRef<html::Div>,
    value: StoredValue<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct ComboboxContext(StoredValue<Combobox>);

#[derive(Clone, Copy)]
struct Combobox {
    base: StoredValue<String>,
    positioner_id: ElementId<html::Div>,
    list_id: ElementId<html::Div>,
    native: NativeOpen<html::Div>,
    value: Controlled<Vec<String>>,
    input_value: Controlled<String>,
    items: Option<Signal<Vec<String>>>,
    labels: Option<Callback<String, String>>,
    filter_query: Memo<String>,
    filtered: Memo<Vec<String>>,
    multiple: Signal<bool>,
    disabled: Signal<bool>,
    read_only: Signal<bool>,
    required: Signal<bool>,
    auto_highlight: Signal<bool>,
    highlight_on_hover: Signal<bool>,
    keep_highlight: Signal<bool>,
    loop_focus: Signal<bool>,
    open_on_input_click: Signal<bool>,
    direction: Signal<Direction>,
    input: NodeRef<html::Input>,
    trigger: NodeRef<html::Button>,
    clear: NodeRef<html::Button>,
    chips: NodeRef<html::Div>,
    // an element of the caller's the popup hangs from, base ui's anchor
    anchor: RwSignal<Option<NodeRef<html::Div>>>,
    positioner: NodeRef<html::Div>,
    popup: NodeRef<html::Div>,
    list: NodeRef<html::Div>,
    inside_popup: RwSignal<bool>,
    entries: StoredValue<Vec<Entry>>,
    next: StoredValue<usize>,
    // the highlighted item's value: base ui's virtual focus, the input keeps the real one
    highlighted: RwSignal<Option<String>>,
    keyboard: StoredValue<bool>,
    // base ui's queryChangedAfterOpen and closeQuery, so a closing popup keeps its filter
    query_changed: RwSignal<bool>,
    close_query: RwSignal<Option<String>>,
    pressed: StoredValue<Option<usize>>,
}

impl ComboboxContext {
    fn combobox(self) -> Combobox {
        self.0.get_value()
    }

    fn try_combobox(self) -> Option<Combobox> {
        self.0.try_get_value()
    }

    fn open(self) -> bool {
        self.try_combobox()
            .is_some_and(|combobox| combobox.native.state().get())
    }

    fn open_untracked(self) -> bool {
        self.try_combobox()
            .is_some_and(|combobox| combobox.native.state().get_untracked())
    }

    fn anchor_name(self) -> String {
        let id = self
            .try_combobox()
            .map(|combobox| combobox.positioner_id.get())
            .unwrap_or_default();
        format!("--{id}")
    }

    fn label(self, value: &str) -> String {
        match self.try_combobox().and_then(|combobox| combobox.labels) {
            Some(labels) => labels.run(value.to_string()),
            None => value.to_string(),
        }
    }

    fn single(self) -> bool {
        self.try_combobox()
            .is_some_and(|combobox| !combobox.multiple.get_untracked())
    }

    fn selected_label(self) -> String {
        let Some(combobox) = self.try_combobox() else {
            return String::new();
        };
        if combobox.multiple.get_untracked() {
            return String::new();
        }
        combobox
            .value
            .with_untracked(|values| values.first().cloned())
            .map(|value| self.label(&value))
            .unwrap_or_default()
    }

    fn chosen(self, value: &str) -> bool {
        self.try_combobox().is_some_and(|combobox| {
            let multiple = combobox.multiple.get();
            combobox.value.with(|values| {
                if multiple {
                    values.iter().any(|chosen| chosen == value)
                } else {
                    values.first().is_some_and(|chosen| chosen == value)
                }
            })
        })
    }

    // base ui's list is empty when the items it was given filter down to none
    fn empty(self) -> bool {
        self.try_combobox()
            .is_none_or(|combobox| match combobox.items {
                Some(_) => combobox.filtered.with(Vec::is_empty),
                None => true,
            })
    }

    fn query_now(self) -> String {
        self.try_combobox()
            .map(|combobox| {
                combobox
                    .input_value
                    .with_untracked(|text| text.trim().to_string())
            })
            .unwrap_or_default()
    }

    fn input(self) -> Option<HtmlElement> {
        self.try_combobox()?
            .input
            .try_get_untracked()
            .flatten()
            .map(HtmlElement::from)
    }

    fn trigger(self) -> Option<HtmlElement> {
        self.try_combobox()?
            .trigger
            .try_get_untracked()
            .flatten()
            .map(HtmlElement::from)
    }

    // what the popup is placed against: the caller's anchor, else the trigger or the input
    fn anchored(self) -> Option<web_sys::Element> {
        let combobox = self.try_combobox()?;
        if let Some(anchor) = combobox.anchor.get_untracked()
            && let Some(element) = anchor.get_untracked()
        {
            return Some(element.into());
        }
        if combobox.inside_popup.get_untracked() {
            self.trigger().map(Into::into)
        } else {
            self.input().map(Into::into)
        }
    }

    // a press on these isn't outside it
    fn owns(self, node: &Node) -> bool {
        let Some(combobox) = self.try_combobox() else {
            return false;
        };
        let parts: [Option<Node>; 5] = [
            combobox
                .positioner
                .try_get_untracked()
                .flatten()
                .map(Into::into),
            combobox.input.try_get_untracked().flatten().map(Into::into),
            combobox
                .trigger
                .try_get_untracked()
                .flatten()
                .map(Into::into),
            combobox.clear.try_get_untracked().flatten().map(Into::into),
            combobox.chips.try_get_untracked().flatten().map(Into::into),
        ];
        parts.iter().flatten().any(|part| part.contains(Some(node)))
    }

    fn register(self, value: String, element: NodeRef<html::Div>) -> usize {
        let combobox = self.combobox();
        let id = combobox
            .next
            .try_update_value(|next| {
                *next += 1;
                *next - 1
            })
            .unwrap_or_default();
        let entry = Entry {
            id,
            element,
            value: StoredValue::new(value),
        };
        combobox.entries.update_value(|entries| entries.push(entry));
        on_cleanup(move || {
            combobox
                .entries
                .try_update_value(|entries| entries.retain(|entry| entry.id != id));
        });
        id
    }

    fn ordered(self) -> Vec<(Entry, HtmlElement)> {
        let mut items: Vec<(Entry, HtmlElement)> = self
            .try_combobox()
            .and_then(|combobox| {
                combobox.entries.try_with_value(|entries| {
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

    // the values the arrows move through, in order: the filtered items, or the rendered ones
    fn visible(self) -> Vec<String> {
        let Some(combobox) = self.try_combobox() else {
            return Vec::new();
        };
        if combobox.items.is_some() {
            return combobox.filtered.get_untracked();
        }
        self.ordered()
            .into_iter()
            .filter_map(|(entry, _)| entry.value.try_get_value())
            .collect()
    }

    fn element_of(self, value: &str) -> Option<HtmlElement> {
        self.ordered()
            .into_iter()
            .find(|(entry, _)| entry.value.with_value(|own| own == value))
            .map(|(_, element)| element)
    }

    fn highlight(self, value: Option<String>, reveal: bool) {
        let Some(combobox) = self.try_combobox() else {
            return;
        };
        if reveal && let Some(element) = value.as_deref().and_then(|value| self.element_of(value)) {
            scroll_into_view(&element);
        }
        combobox.highlighted.try_set(value);
    }

    fn show(self, reason: ChangeReason, opening: Opening) {
        let Some(combobox) = self.try_combobox() else {
            return;
        };
        if combobox.disabled.get_untracked() || combobox.native.state().get_untracked() {
            return;
        }
        if combobox.close_query.get_untracked().is_some() {
            self.reopened(reason == ChangeReason::InputChange);
        }
        combobox.native.state().set(true, reason);
        if !combobox.native.state().get_untracked() {
            return;
        }
        // base ui's focusItemOnOpen: the selected item, else where the arrows asked
        let values = self.visible();
        let target = match opening {
            Opening::Typing => {
                if combobox.auto_highlight.get_untracked() {
                    values.first().cloned()
                } else {
                    None
                }
            }
            _ if combobox.query_changed.get_untracked() => None,
            _ => {
                let selected = values
                    .iter()
                    .find(|value| self.chosen_untracked(value))
                    .cloned();
                match (selected, opening) {
                    (Some(selected), _) => Some(selected),
                    (None, Opening::First) => values.first().cloned(),
                    (None, Opening::Last) => values.last().cloned(),
                    (None, _) => None,
                }
            }
        };
        combobox.highlighted.set(target);
    }

    fn chosen_untracked(self, value: &str) -> bool {
        self.try_combobox().is_some_and(|combobox| {
            let multiple = combobox.multiple.get_untracked();
            combobox.value.with_untracked(|values| {
                if multiple {
                    values.iter().any(|chosen| chosen == value)
                } else {
                    values.first().is_some_and(|chosen| chosen == value)
                }
            })
        })
    }

    // base ui's handleInterruptedReopen: back before the last close had finished
    fn reopened(self, typing: bool) {
        let Some(combobox) = self.try_combobox() else {
            return;
        };
        let text = combobox.input_value.get_untracked();
        let selected = self.selected_label();
        let close_query = combobox.close_query.get_untracked();
        let clears = !typing
            && combobox.inside_popup.get_untracked()
            && !text.is_empty()
            && (close_query.as_deref() == Some(text.trim()) || text == selected);
        let matches_selected =
            self.single() && !combobox.inside_popup.get_untracked() && text == selected;
        if !typing && (clears || text.is_empty() || matches_selected) {
            combobox.query_changed.set(false);
        }
        combobox.close_query.set(None);
        if clears {
            self.set_input(String::new(), ChangeReason::InputClear);
        }
    }

    fn hide(self, reason: ChangeReason) {
        self.hide_after(reason, self.query_now());
    }

    // with the query as it was before whatever closes it changed the input
    fn hide_after(self, reason: ChangeReason, query: String) {
        let Some(combobox) = self.try_combobox() else {
            return;
        };
        if !combobox.native.state().get_untracked() {
            return;
        }
        if combobox.query_changed.get_untracked() {
            let empty = query.is_empty();
            combobox.close_query.set(Some(query));
            if self.single() {
                if empty {
                    combobox.query_changed.set(false);
                }
            } else {
                if combobox.inside_popup.get_untracked() {
                    combobox.highlighted.set(None);
                }
                if !combobox.inside_popup.get_untracked() {
                    self.set_input(String::new(), ChangeReason::InputClear);
                }
            }
        }
        combobox.native.state().set(false, reason);
        // a popup with its own input gives the focus back to the trigger
        if combobox.inside_popup.get_untracked()
            && reason != ChangeReason::FocusOut
            && let Some(trigger) = self.trigger()
        {
            focus(&trigger);
        }
    }

    // once it's hidden: base ui's handleUnmount
    fn closed(self) {
        let Some(combobox) = self.try_combobox() else {
            return;
        };
        combobox.query_changed.try_set(false);
        combobox.close_query.try_set(None);
        combobox.highlighted.try_set(None);
        let text = combobox.input_value.get_untracked();
        if combobox.multiple.get_untracked() || combobox.inside_popup.get_untracked() {
            if !text.is_empty() {
                self.set_input(String::new(), ChangeReason::InputClear);
            }
        } else {
            // whatever was typed and not picked gives way to the selected label
            let label = self.selected_label();
            if text != label {
                let reason = if label.is_empty() {
                    ChangeReason::InputClear
                } else {
                    ChangeReason::None
                };
                self.set_input(label, reason);
            }
        }
    }

    fn set_input(self, text: String, reason: ChangeReason) {
        if let Some(combobox) = self.try_combobox() {
            combobox.input_value.set(text, reason);
        }
    }

    fn set_value(self, next: Vec<String>, reason: ChangeReason) {
        let Some(combobox) = self.try_combobox() else {
            return;
        };
        combobox.value.set(next, reason);
        // an input outside the popup shows the label of what's selected
        if self.single() && !combobox.inside_popup.get_untracked() {
            self.set_input(self.selected_label(), reason);
        }
    }

    // base ui's handleSelection
    fn pick(self, value: String) {
        let Some(combobox) = self.try_combobox() else {
            return;
        };
        if combobox.disabled.get_untracked() || combobox.read_only.get_untracked() {
            return;
        }
        let query = self.query_now();
        if combobox.multiple.get_untracked() {
            let mut next = combobox.value.get_untracked();
            if let Some(at) = next.iter().position(|chosen| *chosen == value) {
                next.remove(at);
            } else {
                next.push(value);
            }
            self.set_value(next, ChangeReason::ItemPress);
            let filtering = self.input().is_some_and(|input| {
                !input
                    .unchecked_ref::<web_sys::HtmlInputElement>()
                    .value()
                    .trim()
                    .is_empty()
            });
            if !filtering {
                return;
            }
            if combobox.inside_popup.get_untracked() {
                self.set_input(String::new(), ChangeReason::InputClear);
            } else {
                self.hide_after(ChangeReason::ItemPress, query);
            }
            return;
        }
        self.set_value(vec![value], ChangeReason::ItemPress);
        self.hide_after(ChangeReason::ItemPress, query);
    }

    // the arrows, with base ui's escape to the input off either end while looping
    fn navigate(self, down: bool) {
        let Some(combobox) = self.try_combobox() else {
            return;
        };
        let values = self.visible();
        let Some(last) = values.len().checked_sub(1) else {
            combobox.highlighted.set(None);
            return;
        };
        let at = combobox
            .highlighted
            .get_untracked()
            .and_then(|value| values.iter().position(|own| *own == value));
        let looping = combobox.loop_focus.get_untracked();
        let escape = looping && !combobox.auto_highlight.get_untracked();
        let end = |wrapped: usize, held: usize| {
            if escape {
                None
            } else if looping {
                Some(wrapped)
            } else {
                Some(held)
            }
        };
        let next = match (at, down) {
            (None, false) if looping => Some(last),
            (None, _) => Some(0),
            (Some(at), true) if at < last => Some(at + 1),
            (Some(_), true) => end(0, last),
            (Some(at), false) if at > 0 => Some(at - 1),
            (Some(_), false) => end(last, 0),
        };
        self.highlight(next.map(|index| values[index].clone()), true);
    }

    fn click_highlighted(self) -> bool {
        let Some(value) = self
            .try_combobox()
            .and_then(|combobox| combobox.highlighted.get_untracked())
        else {
            return false;
        };
        let Some(element) = self.element_of(&value) else {
            return false;
        };
        element.click();
        true
    }

    fn hovered(self, value: &str, event: &ev::MouseEvent) {
        let Some(combobox) = self.try_combobox() else {
            return;
        };
        // webkit sends a move without moving when the page under a still pointer changes
        let still = event.movement_x() == 0 && event.movement_y() == 0;
        if (still && webkit()) || !combobox.native.state().get_untracked() {
            return;
        }
        combobox.keyboard.set_value(false);
        if combobox.highlight_on_hover.get_untracked()
            && combobox
                .highlighted
                .with_untracked(|own| own.as_deref() != Some(value))
        {
            combobox.highlighted.set(Some(value.to_string()));
        }
    }

    fn unhovered(self, event: &ev::PointerEvent) {
        let Some(combobox) = self.try_combobox() else {
            return;
        };
        if !combobox.native.state().get_untracked()
            || combobox.keyboard.get_value()
            || event.pointer_type() == "touch"
            || !combobox.highlight_on_hover.get_untracked()
            || combobox.keep_highlight.get_untracked()
        {
            return;
        }
        let to = event
            .related_target()
            .and_then(|target| target.dyn_into::<Node>().ok());
        let onto_item = to.is_some_and(|to| {
            self.ordered()
                .iter()
                .any(|(_, element)| element.is_same_node(Some(&to)))
        });
        if !onto_item {
            combobox.highlighted.set(None);
        }
    }

    // base ui's ComboboxInput onChange
    fn typed(self, event: &ev::Event) {
        let Some(combobox) = self.try_combobox() else {
            return;
        };
        let input = event_target::<web_sys::HtmlInputElement>(event);
        let text = input.value();
        let kind = event
            .dyn_ref::<web_sys::InputEvent>()
            .map(web_sys::InputEvent::input_type)
            .unwrap_or_default();
        // autofill replaces the text without typing it
        let typed = !kind.is_empty() && kind != "insertReplacementText";
        self.set_input(text.clone(), ChangeReason::InputChange);
        // like react, a controlled input goes back to its value unless the caller took the new one
        let kept = combobox.input_value.get_untracked();
        if kept != text {
            input.set_value(&kept);
        }
        let trimmed = text.trim().to_string();
        if typed {
            if !trimmed.is_empty() {
                combobox.query_changed.set(true);
            }
            if let Some(list) = combobox.list.get_untracked() {
                list.set_scroll_top(0);
            }
            if !trimmed.is_empty()
                && combobox.auto_highlight.get_untracked()
                && combobox.native.state().get_untracked()
            {
                combobox.highlighted.set(self.visible().first().cloned());
            }
        }
        if text.is_empty() && !combobox.inside_popup.get_untracked() {
            if self.single() {
                self.set_value(Vec::new(), ChangeReason::InputClear);
            }
            if !combobox.open_on_input_click.get_untracked() {
                self.hide(ChangeReason::InputClear);
            }
        }
        let editable = !combobox.read_only.get_untracked() && !combobox.disabled.get_untracked();
        if editable && !trimmed.is_empty() && typed && !combobox.native.state().get_untracked() {
            self.show(ChangeReason::InputChange, Opening::Typing);
            if !combobox.auto_highlight.get_untracked() {
                combobox.highlighted.set(None);
            }
        } else if combobox.native.state().get_untracked()
            && !combobox.auto_highlight.get_untracked()
        {
            combobox.highlighted.set(None);
        }
    }

    fn keydown(self, event: &ev::KeyboardEvent, chips: Option<ChipsContext>) {
        let Some(combobox) = self.try_combobox() else {
            return;
        };
        if event.ctrl_key() || event.shift_key() || event.alt_key() || event.meta_key() {
            return;
        }
        combobox.keyboard.set_value(true);
        let key = event.key();
        let open = combobox.native.state().get_untracked();
        if combobox.disabled.get_untracked() || combobox.read_only.get_untracked() {
            if combobox.read_only.get_untracked()
                && key == "Enter"
                && open
                && combobox.highlighted.with_untracked(Option::is_some)
            {
                stop(event);
            }
            return;
        }
        let input = event_target::<web_sys::HtmlInputElement>(event);
        // home and end move the caret, not the highlight
        if key == "Home" || key == "End" {
            stop(event);
            let end = if key == "End" {
                input.value().encode_utf16().count()
            } else {
                0
            };
            let at = u32::try_from(end).unwrap_or(u32::MAX);
            let _ = input.set_selection_range(at, at);
            return;
        }
        if !open && key == "Escape" {
            let clear = combobox.value.with_untracked(Vec::is_empty);
            self.set_input(String::new(), ChangeReason::EscapeKey);
            self.set_value(Vec::new(), ChangeReason::EscapeKey);
            if !clear {
                event.stop_propagation();
            }
            return;
        }
        if let Some(chips) = chips
            && chips.keydown(self, event, &input)
        {
            return;
        }
        match key.as_str() {
            "Enter" if open => {
                if self.click_highlighted() {
                    stop(event);
                } else {
                    self.hide(ChangeReason::None);
                }
            }
            "ArrowDown" | "ArrowUp" => {
                stop(event);
                let down = key == "ArrowDown";
                if open {
                    self.navigate(down);
                } else {
                    let opening = if down { Opening::First } else { Opening::Last };
                    self.show(ChangeReason::ListNavigation, opening);
                    let value = combobox.highlighted.get_untracked();
                    request_animation_frame(move || self.highlight(value, true));
                }
            }
            _ => {}
        }
    }
}

fn webkit() -> bool {
    window()
        .navigator()
        .user_agent()
        .is_ok_and(|agent| agent.contains("AppleWebKit") && !agent.contains("Chrome"))
}

// base ui's filter: the query anywhere in the label, as a searching Intl.Collator of base sensitivity
fn contains(label: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    #[cfg(target_arch = "wasm32")]
    {
        collated(label, query)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        label.to_lowercase().contains(&query.to_lowercase())
    }
}

#[cfg(target_arch = "wasm32")]
fn collated(label: &str, query: &str) -> bool {
    use leptos::wasm_bindgen::JsValue;
    use web_sys::js_sys::{Array, Function, Intl, Object, Reflect};
    thread_local! {
        static COMPARE: Function = {
            let options = Object::new();
            for (key, value) in [
                ("usage", JsValue::from_str("search")),
                ("sensitivity", JsValue::from_str("base")),
                ("ignorePunctuation", JsValue::TRUE),
            ] {
                let _ = Reflect::set(&options, &JsValue::from_str(key), &value);
            }
            Intl::Collator::new(&Array::new(), &options).compare()
        };
    }
    // base ui slides a window as long as the query along the label, in utf-16 units as js counts them
    let units: Vec<u16> = label.encode_utf16().collect();
    let width = query.encode_utf16().count();
    let Some(last) = units.len().checked_sub(width) else {
        return false;
    };
    let query = JsValue::from_str(query);
    COMPARE.with(|compare| {
        (0..=last).any(|start| {
            let window = String::from_utf16_lossy(&units[start..start + width]);
            compare
                .call2(&JsValue::NULL, &JsValue::from_str(&window), &query)
                .ok()
                .and_then(|order| order.as_f64())
                == Some(0.0)
        })
    })
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
pub fn ComboboxRoot(
    #[prop(optional, into)] value: Option<Signal<Vec<String>>>,
    #[prop(optional)] default_value: Vec<String>,
    #[prop(optional, into)] on_value_change: Option<Callback<(Vec<String>, ChangeReason)>>,
    #[prop(optional, into)] input_value: Option<Signal<String>>,
    #[prop(optional, into)] default_input_value: Option<String>,
    #[prop(optional, into)] on_input_value_change: Option<Callback<(String, ChangeReason)>>,
    #[prop(optional, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, ChangeReason)>>,
    #[prop(optional, into)] items: Option<Signal<Vec<String>>>,
    #[prop(optional, into)] item_to_string_label: Option<Callback<String, String>>,
    #[prop(optional, into)] multiple: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] read_only: Signal<bool>,
    #[prop(optional, into)] required: Signal<bool>,
    #[prop(optional, into)] auto_highlight: Signal<bool>,
    #[prop(into, default = true.into())] highlight_item_on_hover: Signal<bool>,
    #[prop(optional, into)] keep_highlight: Signal<bool>,
    #[prop(into, default = true.into())] loop_focus: Signal<bool>,
    #[prop(into, default = true.into())] open_on_input_click: Signal<bool>,
    #[prop(optional, into)] name: Option<Signal<String>>,
    #[prop(optional, into)] form: Option<Signal<String>>,
    #[prop(optional, into)] id: Option<String>,
    children: Children,
) -> impl IntoView {
    let base = id.unwrap_or_else(use_id);
    let (positioner, list) = (NodeRef::new(), NodeRef::new());
    let popup = NodeRef::new();
    let state = Controlled::new(open, default_open, on_open_change);
    let label = move |value: &str| match item_to_string_label {
        Some(labels) => labels.run(value.to_string()),
        None => value.to_string(),
    };
    // a single combobox's input starts on the label of what's selected
    let first_text = default_input_value.unwrap_or_else(|| {
        if multiple.get_untracked() {
            String::new()
        } else {
            default_value
                .first()
                .map(|value| label(value))
                .unwrap_or_default()
        }
    });
    let value = Controlled::new(value, default_value, on_value_change);
    let text = Controlled::new(input_value, first_text, on_input_value_change);
    let query_changed = RwSignal::new(false);
    let close_query = RwSignal::new(None::<String>);
    let query = Memo::new(move |_| match close_query.get() {
        Some(query) if !state.get() => query,
        _ => text.with(|text| text.trim().to_string()),
    });
    // a single combobox showing its selected label shows every item, as base ui's
    let filter_query = Memo::new(move |_| {
        let query = query.get();
        if multiple.get() || query_changed.get() || query.is_empty() {
            return query;
        }
        let selected = value
            .with(|values| values.first().map(|value| label(value)))
            .unwrap_or_default();
        let bypass = selected.encode_utf16().count() == query.encode_utf16().count()
            && contains(&selected, &query);
        if bypass { String::new() } else { query }
    });
    let filtered = Memo::new(move |_| {
        let query = filter_query.get();
        items.map_or_else(Vec::new, |items| {
            items.with(|items| {
                items
                    .iter()
                    .filter(|item| contains(&label(item), &query))
                    .cloned()
                    .collect()
            })
        })
    });
    let context = ComboboxContext(StoredValue::new(Combobox {
        positioner_id: ElementId::new(positioner, format!("{base}-positioner")),
        list_id: ElementId::new(list, format!("{base}-list")),
        base: StoredValue::new(base),
        native: NativeOpen::new(Native::Popover, positioner, state, Some(popup)),
        value,
        input_value: text,
        items,
        labels: item_to_string_label,
        filter_query,
        filtered,
        multiple,
        disabled,
        read_only,
        required,
        auto_highlight,
        highlight_on_hover: highlight_item_on_hover,
        keep_highlight,
        loop_focus,
        open_on_input_click,
        direction: use_direction(),
        input: NodeRef::new(),
        trigger: NodeRef::new(),
        clear: NodeRef::new(),
        chips: NodeRef::new(),
        anchor: RwSignal::new(None),
        positioner,
        popup,
        list,
        inside_popup: RwSignal::new(false),
        entries: StoredValue::new(Vec::new()),
        next: StoredValue::new(0),
        highlighted: RwSignal::new(None),
        keyboard: StoredValue::new(true),
        query_changed,
        close_query,
        pressed: StoredValue::new(None),
    }));
    dismiss(
        move || context.open(),
        move |node| context.owns(node),
        move |reason| context.hide(reason),
    );
    // what a form gets: one value, or each of several
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

// base ui's visually hidden dismiss button, for screen readers to close the popup
fn dismiss_button(context: ComboboxContext) -> impl IntoView {
    view! {
        <span
            role="button"
            aria-label="Dismiss"
            style="clip-path:inset(50%);overflow:hidden;white-space:nowrap;border:0;padding:0;width:1px;height:1px;margin:-1px;position:absolute"
            on:click=move |_| context.hide(ChangeReason::ClosePress)
        ></span>
    }
}

#[component(element = "input", skip(r#type, role, id, autocomplete))]
pub fn ComboboxInput(
    #[prop(optional_no_strip, into)] disabled: Option<Signal<bool>>,
    #[prop(optional_no_strip, into)] id: Option<String>,
) -> impl IntoView {
    let context = expect_context::<ComboboxContext>();
    let combobox = context.combobox();
    let inside = use_context::<Placement>().is_some();
    combobox.inside_popup.set(inside);
    if inside {
        combobox.input_value.set(String::new(), ChangeReason::None);
    }
    let chips = use_context::<ChipsContext>();
    let disabled =
        move || combobox.disabled.get() || disabled.is_some_and(|disabled| disabled.get());
    let open = move || context.open();
    let id = id.or_else(|| (!inside).then(|| combobox.base.get_value()));
    let active = move || {
        if !open() {
            return None;
        }
        let value = combobox.highlighted.get()?;
        let element = context.element_of(&value)?;
        Some(element.id())
    };
    view! {
        {move || (open() && !inside).then(|| dismiss_button(context))}
        <input
            {..attrs}
            node_ref=combobox.input
            id=id
            role="combobox"
            autocomplete="off"
            spellcheck="false"
            {..leptos::attr::custom::custom_attribute("autocorrect", "off")}
            autocapitalize="none"
            // a property never reaches the server's html, so the attribute carries the first value
            value=combobox.input_value.get_untracked()
            prop:value=move || combobox.input_value.get()
            aria-expanded=move || open().to_string()
            aria-haspopup="listbox"
            aria-controls=move || open().then(|| combobox.list_id.get())
            aria-autocomplete=move || if combobox.read_only.get() { "none" } else { "list" }
            aria-activedescendant=active
            aria-readonly=move || combobox.read_only.get().then_some("true")
            aria-required=move || combobox.required.get().then_some("true")
            disabled=disabled
            readonly=move || combobox.read_only.get()
            data-popup-open=move || open().then_some("")
            data-pressed=move || open().then_some("")
            data-disabled=move || disabled().then_some("")
            data-readonly=move || combobox.read_only.get().then_some("")
            data-list-empty=move || context.empty().then_some("")
            on:input=move |event: ev::Event| context.typed(&event)
            on:keydown=move |event: ev::KeyboardEvent| context.keydown(&event, chips)
            on:mousedown=move |event: ev::MouseEvent| {
                if event.button() != 0 || disabled() || !combobox.open_on_input_click.get_untracked() {
                    return;
                }
                context.show(ChangeReason::InputPress, Opening::Pointer);
            }
            on:pointermove=move |_| combobox.keyboard.set_value(false)
            on:pointerdown=move |_| combobox.keyboard.set_value(false)
            on:focusout=move |event: ev::FocusEvent| {
                let to = event.related_target().and_then(|target| target.dyn_into::<Node>().ok());
                if to.is_some_and(|to| !context.owns(&to)) {
                    context.hide(ChangeReason::FocusOut);
                }
            }
        />
    }
}

#[component(element = "button", skip(r#type, id, role))]
pub fn ComboboxTrigger(
    #[prop(optional_no_strip, into)] disabled: Option<Signal<bool>>,
    #[prop(optional_no_strip, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<ComboboxContext>();
    let combobox = context.combobox();
    let disabled =
        move || combobox.disabled.get() || disabled.is_some_and(|disabled| disabled.get());
    let open = move || context.open();
    let inside = move || combobox.inside_popup.get();
    let tabindex = Signal::derive(move || if inside() { 0 } else { -1 });
    let touch = StoredValue::new(false);
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
        let target = event
            .target()
            .and_then(|target| target.dyn_into::<Node>().ok());
        if target.is_some_and(|target| context.owns(&target)) {
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
    let controls = move || {
        if !open() {
            return None;
        }
        Some(if inside() {
            format!("{}-popup", combobox.base.get_value())
        } else {
            combobox.list_id.get()
        })
    };
    view! {
        <button
            {..attrs}
            {..button_attributes(disabled, || false, Some(tabindex))}
            node_ref=combobox.trigger
            id=move || if inside() { id.clone().or_else(|| Some(combobox.base.get_value())) } else { id.clone() }
            type="button"
            role=move || inside().then_some("combobox")
            aria-expanded=move || open().to_string()
            aria-haspopup=move || if inside() { "dialog" } else { "listbox" }
            aria-controls=controls
            aria-required=move || (inside() && combobox.required.get()).then_some("true")
            aria-readonly=move || (inside() && combobox.read_only.get()).then_some("true")
            data-popup-open=move || open().then_some("")
            data-pressed=move || open().then_some("")
            data-placeholder=move || combobox.value.with(Vec::is_empty).then_some("")
            data-readonly=move || combobox.read_only.get().then_some("")
            data-list-empty=move || context.empty().then_some("")
            on:pointerdown=move |event: ev::PointerEvent| touch.set_value(event.pointer_type() == "touch")
            on:mousedown=move |event: ev::MouseEvent| {
                if disabled() || event.button() != 0 {
                    return;
                }
                if !touch.get_value() {
                    if let Some(input) = context.input() {
                        focus(&input);
                    }
                    // the focus stays in the input
                    if !inside() {
                        event.prevent_default();
                    }
                }
                if context.open_untracked() {
                    context.hide(ChangeReason::TriggerPress);
                    return;
                }
                context.show(ChangeReason::TriggerPress, Opening::Pointer);
                if inside() {
                    release.set_value(Some(window_event_listener(ev::mouseup, released)));
                }
            }
            on:click=move |event: ev::MouseEvent| {
                if event.detail() != 0 || disabled() {
                    return;
                }
                if context.open_untracked() {
                    context.hide(ChangeReason::TriggerPress);
                } else {
                    context.show(ChangeReason::TriggerPress, Opening::First);
                }
            }
            on:keydown=move |event: ev::KeyboardEvent| {
                let key = event.key();
                if key == "ArrowDown" || key == "ArrowUp" {
                    stop(&event);
                    let opening = if key == "ArrowUp" { Opening::Last } else { Opening::First };
                    context.show(ChangeReason::ListNavigation, opening);
                    if let Some(input) = context.input() {
                        focus(&input);
                    }
                }
            }
        >
            {children.map(|children| children())}
        </button>
    }
}

#[component(element = "button", skip(r#type))]
pub fn ComboboxClear(
    #[prop(optional_no_strip, into)] disabled: Option<Signal<bool>>,
    #[prop(optional)] keep_mounted: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<ComboboxContext>();
    let combobox = context.combobox();
    let disabled =
        move || combobox.disabled.get() || disabled.is_some_and(|disabled| disabled.get());
    let visible = move || !combobox.value.with(Vec::is_empty);
    let inner = match children {
        Some(children) => Either::Left(children()),
        None => Either::Right("x"),
    };
    // hidden rather than unmounted: leptos's dev builds drop a caller's attr: from an element that comes back
    view! {
        <button
            {..attrs}
            {..button_attributes(disabled, || false, Some(Signal::stored(-1)))}
            node_ref=combobox.clear
            type="button"
            hidden=move || !keep_mounted && !visible()
            data-visible=move || visible().then_some("")
            data-popup-open=move || context.open().then_some("")
            on:mousedown=move |event: ev::MouseEvent| event.prevent_default()
            on:click=move |_| {
                if disabled() || combobox.read_only.get_untracked() {
                    return;
                }
                context.set_input(String::new(), ChangeReason::ClearPress);
                context.set_value(Vec::new(), ChangeReason::ClearPress);
                combobox.highlighted.set(None);
                if let Some(input) = context.input() {
                    focus(&input);
                }
            }
        >
            {inner}
        </button>
    }
}

// what a value's children get: the selected values
pub struct ComboboxValueChildren(Box<dyn Fn(Vec<String>) -> AnyView + Send + Sync>);

impl fmt::Debug for ComboboxValueChildren {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ComboboxValueChildren")
    }
}

impl<F, V> From<F> for ComboboxValueChildren
where
    F: Fn(Vec<String>) -> V + Send + Sync + 'static,
    V: IntoView + 'static,
{
    fn from(children: F) -> Self {
        Self(Box::new(move |values| children(values).into_any()))
    }
}

#[leptos::component]
pub fn ComboboxValue(
    #[prop(optional, into)] placeholder: MaybeProp<String>,
    #[prop(optional, into)] children: Option<ComboboxValueChildren>,
) -> impl IntoView {
    let context = expect_context::<ComboboxContext>();
    let combobox = context.combobox();
    move || {
        let values = combobox.value.get();
        if let Some(children) = &children {
            return Either::Left((children.0)(values));
        }
        if values.is_empty()
            && let Some(placeholder) = placeholder.get()
        {
            return Either::Right(placeholder);
        }
        Either::Right(
            values
                .iter()
                .map(|value| context.label(value))
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

#[leptos::component]
pub fn ComboboxPortal(children: Children) -> impl IntoView {
    children()
}

#[component(element = "div", skip(id, style, role))]
pub fn ComboboxPositioner(
    #[prop(optional, into)] side: Signal<Side>,
    #[prop(optional, into)] align: Signal<Align>,
    #[prop(optional, into)] side_offset: Signal<f64>,
    #[prop(optional, into)] align_offset: Signal<f64>,
    #[prop(optional_no_strip)] anchor: Option<NodeRef<html::Div>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<ComboboxContext>();
    let combobox = context.combobox();
    if anchor.is_some() {
        combobox.anchor.set(anchor);
    }
    let placed = Placement::new(side, align);
    let open = move || context.open();
    let style = move || {
        let inert = if open() { "" } else { ";pointer-events:none" };
        let place = placement(
            &context.anchor_name(),
            side.get(),
            align.get(),
            side_offset.get(),
            align_offset.get(),
        );
        format!("{place};{}{inert}", (placed.sizes())())
    };
    // the anchor takes its name as the popup opens, after hydration: which element it is can change
    let settle = move || {
        if let Some(anchor) = context.anchored() {
            let _ = anchor
                .unchecked_ref::<HtmlElement>()
                .style()
                .set_property("anchor-name", &context.anchor_name());
        }
        if let (Some(anchor), Some(Some(positioner))) =
            (context.anchored(), combobox.positioner.try_get_untracked())
        {
            placed.settle(&anchor, &positioner);
        }
    };
    Effect::new(move || {
        if combobox.native.state().get() {
            // once the press that opened it is done, before the first paint
            request_animation_frame(move || {
                settle();
                let highlighted = combobox.highlighted.get_untracked();
                if let Some(element) = highlighted
                    .as_deref()
                    .and_then(|value| context.element_of(value))
                {
                    scroll_into_view(&element);
                }
                if combobox.inside_popup.get_untracked()
                    && let Some(input) = context.input()
                {
                    focus(&input);
                }
            });
        }
    });
    // filtering changes its height, which can send it to the other side
    Effect::new(move || {
        if let Some(popup) = combobox.popup.get() {
            observe_resize(&popup, move || {
                if context.open_untracked() {
                    settle();
                }
            });
        }
    });
    let empty = move || context.empty();
    view! {
        <div
            {..attrs}
            node_ref=combobox.positioner
            id=combobox.positioner_id.get()
            popover="manual"
            role="presentation"
            style=style
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-side=move || placed.side().as_str()
            data-align=keyword(placed.align)
            data-empty=move || empty().then_some("")
            on:toggle=move |_| {
                combobox.native.sync(ChangeReason::None);
                let hidden = combobox
                    .positioner
                    .get_untracked()
                    .is_some_and(|positioner| !positioner.matches(":popover-open").unwrap_or(false));
                if hidden && !context.open_untracked() {
                    context.closed();
                    placed.reset();
                }
            }
        >
            <Provider value=placed>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "div", skip(id, role, tabindex))]
pub fn ComboboxPopup(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<ComboboxContext>();
    let placed = expect_context::<Placement>();
    let combobox = context.combobox();
    let open = move || context.open();
    let inside = move || combobox.inside_popup.get();
    view! {
        <div
            {..attrs}
            node_ref=combobox.popup
            id=move || inside().then(|| format!("{}-popup", combobox.base.get_value()))
            role=move || if inside() { "dialog" } else { "presentation" }
            tabindex="-1"
            style:position="relative"
            style:left=placed.nudge(|(x, _)| x)
            style:top=placed.nudge(|(_, y)| y)
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-side=move || placed.side().as_str()
            data-align=keyword(placed.align)
            data-empty=move || context.empty().then_some("")
            // the focus goes back to the input, the list only highlights
            on:focusin=move |event: ev::FocusEvent| {
                let target = event.target().and_then(|target| target.dyn_into::<Node>().ok());
                let list = combobox.list.get_untracked().map(Node::from);
                let popup = combobox.popup.get_untracked().map(Node::from);
                let ours = target.is_some_and(|target| {
                    list.is_some_and(|list| list.contains(Some(&target)))
                        || popup.is_some_and(|popup| popup.is_same_node(Some(&target)))
                });
                if ours && let Some(input) = context.input() {
                    focus(&input);
                }
            }
        >
            {children.map(|children| children())}
        </div>
        {move || (open() && !inside()).then(|| dismiss_button(context))}
    }
}

#[component(element = "div", skip(id, role, tabindex))]
pub fn ComboboxList(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<ComboboxContext>();
    let combobox = context.combobox();
    // with the input inside the popup the trigger isn't typed in, so the list names the highlight
    let active = move || {
        if !combobox.inside_popup.get() || !context.open() {
            return None;
        }
        let value = combobox.highlighted.get()?;
        Some(context.element_of(&value)?.id())
    };
    view! {
        <div
            {..attrs}
            node_ref=combobox.list
            id=combobox.list_id.get()
            role="listbox"
            tabindex="-1"
            aria-multiselectable=move || combobox.multiple.get().then_some("true")
            aria-readonly=move || combobox.read_only.get().then_some("true")
            aria-activedescendant=active
            data-empty=move || context.empty().then_some("")
            on:keydown=move |event: ev::KeyboardEvent| {
                if event.key() == "Enter"
                    && !combobox.disabled.get_untracked()
                    && !combobox.read_only.get_untracked()
                    && context.click_highlighted()
                {
                    stop(&event);
                }
            }
            on:pointermove=move |_| combobox.keyboard.set_value(false)
        >
            {children.map(|children| children())}
        </div>
    }
}

// the items a group shows, when it was given them
#[derive(Clone, Copy)]
struct GroupItems(Memo<Vec<String>>);

#[leptos::component]
pub fn ComboboxCollection<F, V>(children: F) -> impl IntoView
where
    F: Fn(String) -> V + Clone + Send + Sync + 'static,
    V: IntoView + 'static,
{
    let context = expect_context::<ComboboxContext>();
    let combobox = context.combobox();
    let group = use_context::<GroupItems>();
    let each = move || match group {
        Some(GroupItems(items)) => items.get(),
        None => combobox.filtered.get(),
    };
    view! { <For each=each key=Clone::clone children=children /> }
}

#[component(element = "div", skip(role))]
pub fn ComboboxEmpty(#[prop(optional)] children: Option<ChildrenFn>) -> impl IntoView {
    let context = expect_context::<ComboboxContext>();
    view! {
        <div {..attrs} role="status" aria-live="polite" aria-atomic="true">
            {move || {
                context
                    .empty()
                    .then(|| children.clone().map(|children| children()))
                    .flatten()
            }}
        </div>
    }
}

#[derive(Clone, Copy)]
struct ItemContext {
    selected: Signal<bool>,
}

#[component(element = "div", skip(id, role, tabindex))]
pub fn ComboboxItem(
    #[prop(into)] value: String,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<ComboboxContext>();
    let combobox = context.combobox();
    let element = NodeRef::new();
    let disabled = Signal::derive(move || disabled.get() || combobox.disabled.get());
    let id = context.register(value.clone(), element);
    let value = StoredValue::new(value);
    let selected = Signal::derive(move || value.with_value(|value| context.chosen(value)));
    let highlighted = move || {
        combobox
            .highlighted
            .with(|own| value.with_value(|value| own.as_deref() == Some(value.as_str())))
    };
    view! {
        <div
            {..attrs}
            node_ref=element
            id=format!("{}-{id}", combobox.base.get_value())
            role="option"
            aria-selected=move || selected.get().to_string()
            aria-disabled=move || disabled.get().then_some("true")
            data-selected=move || selected.get().then_some("")
            data-highlighted=move || highlighted().then_some("")
            data-disabled=move || disabled.get().then_some("")
            // the input keeps the focus
            on:pointerdown=move |event: ev::PointerEvent| {
                if event.is_primary() {
                    combobox.pressed.set_value(Some(id));
                }
                event.prevent_default();
            }
            on:mousedown=move |event: ev::MouseEvent| event.prevent_default()
            on:click=move |_| {
                if !disabled.get_untracked() && !combobox.read_only.get_untracked() {
                    context.pick(value.get_value());
                }
            }
            // a press that began elsewhere, dragged onto a highlighted item, picks it
            on:mouseup=move |event: ev::MouseEvent| {
                let started = combobox.pressed.try_update_value(Option::take).flatten() == Some(id);
                if disabled.get_untracked()
                    || combobox.read_only.get_untracked()
                    || event.button() != 0
                    || started
                    || !highlighted()
                {
                    return;
                }
                context.pick(value.get_value());
            }
            on:mousemove=move |event: ev::MouseEvent| value.with_value(|value| context.hovered(value, &event))
            on:pointerleave=move |event: ev::PointerEvent| context.unhovered(&event)
        >
            <Provider value=ItemContext { selected }>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "span")]
pub fn ComboboxItemIndicator(
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
pub fn ComboboxGroup(
    #[prop(optional_no_strip, into)] items: Option<Signal<Vec<String>>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<ComboboxContext>();
    let combobox = context.combobox();
    let label = RwSignal::new(None::<ElementId<html::Div>>);
    let filtered = items.map(|items| {
        Memo::new(move |_| {
            let query = combobox.filter_query.get();
            items.with(|items| {
                items
                    .iter()
                    .filter(|item| contains(&context.label(item), &query))
                    .cloned()
                    .collect::<Vec<_>>()
            })
        })
    });
    // base ui leaves out a group none of whose items match
    let gone = move || filtered.is_some_and(|items| items.with(Vec::is_empty));
    view! {
        <div
            {..attrs}
            role="group"
            hidden=gone
            aria-labelledby=move || label.get().map(|label| label.get())
        >
            <Provider value=GroupLabel(label)>
                {match filtered {
                    Some(items) => Either::Left(view! {
                        <Provider value=GroupItems(items)>{children.map(|children| children())}</Provider>
                    }),
                    None => Either::Right(children.map(|children| children())),
                }}
            </Provider>
        </div>
    }
}

#[component(element = "div", skip(id))]
pub fn ComboboxGroupLabel(
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
pub fn ComboboxSeparator(
    #[prop(into, default = Orientation::Horizontal.into())] orientation: Signal<Orientation>,
) -> impl IntoView {
    view! { <div {..attrs} role="presentation" data-orientation=keyword(orientation)></div> }
}

#[derive(Clone, Copy)]
struct ChipsContext {
    chips: StoredValue<Vec<(usize, NodeRef<html::Div>)>>,
    next: StoredValue<usize>,
    highlighted: RwSignal<Option<usize>>,
}

impl ChipsContext {
    fn ordered(self) -> Vec<(usize, HtmlElement)> {
        let mut chips: Vec<(usize, HtmlElement)> = self
            .chips
            .try_with_value(|chips| {
                chips
                    .iter()
                    .filter_map(|(id, chip)| {
                        Some((*id, chip.try_get_untracked().flatten()?.into()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        chips.sort_by(|(_, a), (_, b)| document_order(a, b));
        chips
    }

    fn index_of(self, id: usize) -> Option<usize> {
        self.ordered().iter().position(|(chip, _)| *chip == id)
    }

    fn go(self, index: Option<usize>, context: ComboboxContext) {
        self.highlighted.set(index);
        let chip = index.and_then(|index| self.ordered().get(index).map(|(_, chip)| chip.clone()));
        match chip {
            Some(chip) => focus(&chip),
            None => {
                if let Some(input) = context.input() {
                    focus(&input);
                }
            }
        }
    }

    // base ui's chip keys on the input: backspace takes the last chip, the arrow back moves onto them
    fn keydown(
        self,
        context: ComboboxContext,
        event: &ev::KeyboardEvent,
        input: &web_sys::HtmlInputElement,
    ) -> bool {
        let Some(combobox) = context.try_combobox() else {
            return false;
        };
        let key = event.key();
        let count = combobox.value.with_untracked(Vec::len);
        let rendered = self.ordered().len();
        let (back, _) = chip_keys(combobox.direction.get_untracked());
        if key == "Backspace" && input.value().is_empty() && count > 0 {
            let at = if rendered > 0 {
                rendered - 1
            } else {
                count - 1
            };
            let mut next = combobox.value.get_untracked();
            next.remove(at.min(count - 1));
            combobox.highlighted.set(None);
            context.set_value(next, ChangeReason::None);
            return true;
        }
        if key == back && input.selection_start().ok().flatten().unwrap_or(0) == 0 && count > 0 {
            event.prevent_default();
            self.go(rendered.checked_sub(1), context);
            return true;
        }
        false
    }
}

fn chip_keys(direction: Direction) -> (&'static str, &'static str) {
    match direction {
        Direction::Rtl => ("ArrowRight", "ArrowLeft"),
        Direction::Ltr => ("ArrowLeft", "ArrowRight"),
    }
}

fn after_removal(index: usize, count: usize) -> Option<usize> {
    if index + 1 >= count {
        count.checked_sub(2)
    } else {
        Some(index)
    }
}

#[component(element = "div", skip(role))]
pub fn ComboboxChips(
    #[prop(optional_no_strip)] node_ref: Option<NodeRef<html::Div>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<ComboboxContext>();
    let combobox = context.combobox();
    let element = node_ref.unwrap_or(combobox.chips);
    let chips = ChipsContext {
        chips: StoredValue::new(Vec::new()),
        next: StoredValue::new(0),
        highlighted: RwSignal::new(None),
    };
    view! {
        <div
            {..attrs}
            node_ref=element
            role=move || (!combobox.value.with(Vec::is_empty)).then_some("toolbar")
            // a press on the box, not a chip or a button in it, goes to the input
            on:mousedown=move |event: ev::MouseEvent| {
                let target = event.target().and_then(|target| target.dyn_into::<web_sys::Element>().ok());
                let current = event.current_target().and_then(|own| own.dyn_into::<web_sys::Element>().ok());
                let own = target.as_ref().zip(current.as_ref()).is_some_and(|(target, current)| target == current);
                let interactive = target.as_ref().is_some_and(|target| {
                    target.closest("button, input, a, select, textarea, [role=button]").ok().flatten().is_some()
                });
                if !own && interactive {
                    return;
                }
                event.prevent_default();
                if combobox.disabled.get_untracked() {
                    return;
                }
                if let Some(input) = context.input() {
                    focus(&input);
                }
                if combobox.open_on_input_click.get_untracked() {
                    context.show(ChangeReason::InputPress, Opening::Pointer);
                }
            }
        >
            <Provider value=chips>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[derive(Clone, Copy)]
struct ChipContext {
    id: usize,
}

#[component(element = "div", skip(tabindex))]
pub fn ComboboxChip(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<ComboboxContext>();
    let combobox = context.combobox();
    let chips = expect_context::<ChipsContext>();
    let element = NodeRef::new();
    let id = chips
        .next
        .try_update_value(|next| {
            *next += 1;
            *next - 1
        })
        .unwrap_or_default();
    chips.chips.update_value(|all| all.push((id, element)));
    on_cleanup(move || {
        chips
            .chips
            .try_update_value(|all| all.retain(|(chip, _)| *chip != id));
    });
    let keydown = move |event: ev::KeyboardEvent| {
        if combobox.disabled.get_untracked() || combobox.read_only.get_untracked() {
            return;
        }
        let Some(index) = chips.index_of(id) else {
            return;
        };
        let key = event.key();
        let count = chips.ordered().len();
        let (back, forth) = chip_keys(combobox.direction.get_untracked());
        let mut removed = false;
        let next = if key == back {
            event.prevent_default();
            index.checked_sub(1)
        } else if key == forth {
            event.prevent_default();
            (index + 1 < count).then_some(index + 1)
        } else if key == "Backspace" || key == "Delete" {
            stop(&event);
            let total = combobox.value.with_untracked(Vec::len);
            combobox.highlighted.set(None);
            let mut next = combobox.value.get_untracked();
            if index < next.len() {
                next.remove(index);
            }
            context.set_value(next, ChangeReason::None);
            removed = true;
            after_removal(index, total)
        } else if key == "Enter" || key == " " {
            stop(&event);
            None
        } else if key == "ArrowDown" || key == "ArrowUp" {
            stop(&event);
            context.show(ChangeReason::ListNavigation, Opening::First);
            None
        } else {
            None
        };
        // the chips left render with the next frame
        if removed {
            request_animation_frame(move || chips.go(next, context));
        } else {
            chips.go(next, context);
        }
    };
    view! {
        <div
            {..attrs}
            node_ref=element
            tabindex="-1"
            aria-disabled=move || combobox.disabled.get().then_some("true")
            aria-readonly=move || combobox.read_only.get().then_some("true")
            data-disabled=move || combobox.disabled.get().then_some("")
            on:keydown=keydown
        >
            <Provider value=ChipContext { id }>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "button", skip(r#type))]
pub fn ComboboxChipRemove(
    #[prop(optional_no_strip, into)] disabled: Option<Signal<bool>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<ComboboxContext>();
    let combobox = context.combobox();
    let chips = expect_context::<ChipsContext>();
    let chip = expect_context::<ChipContext>();
    let disabled = move || {
        combobox.disabled.get()
            || combobox.read_only.get()
            || disabled.is_some_and(|disabled| disabled.get())
    };
    let remove = move || {
        let Some(index) = chips.index_of(chip.id) else {
            return;
        };
        let mut next = combobox.value.get_untracked();
        if index >= next.len() {
            return;
        }
        let removed = next.remove(index);
        if combobox
            .highlighted
            .with_untracked(|own| own.as_deref() == Some(removed.as_str()))
        {
            combobox.highlighted.set(None);
        }
        context.set_value(next, ChangeReason::ChipRemovePress);
        if let Some(input) = context.input() {
            focus(&input);
        }
    };
    view! {
        <button
            {..attrs}
            {..button_attributes(disabled, || true, Some(Signal::stored(-1)))}
            type="button"
            on:mousedown=move |event: ev::MouseEvent| event.prevent_default()
            on:click=move |event: ev::MouseEvent| {
                event.stop_propagation();
                remove();
            }
            on:keydown=move |event: ev::KeyboardEvent| {
                let key = event.key();
                if key == "Enter" || key == " " {
                    stop(&event);
                    remove();
                }
            }
        >
            {children.map(|children| children())}
        </button>
    }
}
