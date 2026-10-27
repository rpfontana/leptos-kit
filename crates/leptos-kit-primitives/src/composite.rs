use crate::native::document_order;
use crate::{Direction, Orientation, use_direction};
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, Node};

#[derive(Clone, Copy)]
struct Item {
    id: usize,
    disabled: Signal<bool>,
    active: Signal<bool>,
    element: NodeRef<html::Button>,
}

// base ui's composite: one tab stop, arrows and home/end between the items
#[derive(Clone, Copy)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent switches, each set by its own builder method"
)]
pub(crate) struct Composite {
    orientation: Signal<Orientation>,
    loop_focus: Signal<bool>,
    // radio groups take the arrows both ways
    both: bool,
    home_end: bool,
    allow_shift: bool,
    focus_disabled: bool,
    direction: Signal<Direction>,
    // the highlighted item and the active one when it was, so a new active item takes over
    highlighted: RwSignal<Option<(usize, Option<usize>)>>,
    items: RwSignal<Vec<Item>>,
    next: StoredValue<usize>,
}

impl Composite {
    pub(crate) fn new(orientation: Signal<Orientation>, loop_focus: Signal<bool>) -> Self {
        Self {
            orientation,
            loop_focus,
            both: false,
            home_end: false,
            allow_shift: false,
            focus_disabled: false,
            direction: use_direction(),
            highlighted: RwSignal::new(None),
            items: RwSignal::new(Vec::new()),
            next: StoredValue::new(0),
        }
    }

    pub(crate) fn both(mut self) -> Self {
        self.both = true;
        self
    }

    pub(crate) fn home_end(mut self) -> Self {
        self.home_end = true;
        self
    }

    pub(crate) fn allow_shift(mut self) -> Self {
        self.allow_shift = true;
        self
    }

    pub(crate) fn focus_disabled(mut self) -> Self {
        self.focus_disabled = true;
        self
    }

    pub(crate) fn item(
        self,
        disabled: Signal<bool>,
        active: Signal<bool>,
        element: NodeRef<html::Button>,
    ) -> usize {
        let id = self
            .next
            .try_update_value(|next| {
                *next += 1;
                *next - 1
            })
            .unwrap_or_default();
        self.items.update(|items| {
            items.push(Item {
                id,
                disabled,
                active,
                element,
            });
        });
        on_cleanup(move || {
            self.items
                .try_update(|items| items.retain(|item| item.id != id));
        });
        id
    }

    fn focusable(&self, item: &Item) -> bool {
        self.focus_disabled || !item.disabled.get()
    }

    fn active(&self, items: &[Item]) -> Option<usize> {
        items
            .iter()
            .find(|item| self.focusable(item) && item.active.get())
            .map(|item| item.id)
    }

    // the highlighted item, else the active one, else the first that can take focus
    fn current(&self) -> Option<usize> {
        self.items.with(|items| {
            let active = self.active(items);
            let highlighted = self.highlighted.get().filter(|(id, during)| {
                *during == active
                    && items
                        .iter()
                        .any(|item| item.id == *id && self.focusable(item))
            });
            highlighted.map(|(id, _)| id).or(active).or_else(|| {
                items
                    .iter()
                    .find(|item| self.focusable(item))
                    .map(|item| item.id)
            })
        })
    }

    pub(crate) fn tabindex(self, id: usize) -> Signal<i32> {
        Signal::derive(move || if self.current() == Some(id) { 0 } else { -1 })
    }

    pub(crate) fn highlight(self, id: usize) {
        let active = self
            .items
            .with_untracked(|items| untrack(|| self.active(items)));
        self.highlighted.set(Some((id, active)));
    }

    pub(crate) fn item_attributes(self, id: usize) -> impl leptos::attr::Attribute {
        view! { <{..} on:focus=move |_| self.highlight(id) /> }
    }

    pub(crate) fn root_attributes(self) -> impl leptos::attr::Attribute {
        view! { <{..} on:keydown=move |event: ev::KeyboardEvent| self.navigate(&event) /> }
    }

    pub(crate) fn navigate(&self, event: &ev::KeyboardEvent) {
        let key = event.key();
        let modified = (event.shift_key() && !self.allow_shift)
            || event.ctrl_key()
            || event.alt_key()
            || event.meta_key();
        if modified {
            return;
        }
        let rtl = self.direction.get_untracked() == Direction::Rtl;
        let orientation = self.orientation.get_untracked();
        let (forward_key, backward_key) = if rtl {
            ("ArrowLeft", "ArrowRight")
        } else {
            ("ArrowRight", "ArrowLeft")
        };
        let across = self.both || orientation == Orientation::Horizontal;
        let down = self.both || orientation == Orientation::Vertical;
        let forward = (across && key == forward_key) || (down && key == "ArrowDown");
        let backward = (across && key == backward_key) || (down && key == "ArrowUp");
        let home_end = self.home_end && (key == "Home" || key == "End");
        if !forward && !backward && !home_end {
            return;
        }

        let mut elements: Vec<HtmlButtonElement> = self.items.with_untracked(|items| {
            items
                .iter()
                .filter_map(|item| item.element.get_untracked())
                .collect()
        });
        elements.sort_by(|a, b| document_order(a, b));
        let target = event
            .target()
            .and_then(|target| target.dyn_into::<Node>().ok());
        let Some(current) = elements.iter().position(|element| {
            target
                .as_ref()
                .is_some_and(|target| element.contains(Some(target)))
        }) else {
            return;
        };
        let enabled: Vec<usize> = (0..elements.len())
            .filter(|&index| {
                let element = &elements[index];
                self.focus_disabled
                    || (!element.has_attribute("disabled")
                        && element.get_attribute("aria-disabled").as_deref() != Some("true"))
            })
            .collect();
        let (Some(&first), Some(&last)) = (enabled.first(), enabled.last()) else {
            return;
        };
        let looping = self.loop_focus.get_untracked();
        let next = if key == "Home" {
            first
        } else if key == "End" {
            last
        } else if forward {
            match enabled.iter().find(|&&index| index > current) {
                Some(&index) => index,
                None if looping => first,
                None => current,
            }
        } else {
            match enabled.iter().rev().find(|&&index| index < current) {
                Some(&index) => index,
                None if looping => last,
                None => current,
            }
        };
        if next != current {
            event.prevent_default();
            let _ = elements[next].focus();
        }
    }
}
