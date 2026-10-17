use crate::use_direction;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, Node};

#[derive(Clone, Copy)]
struct Item {
    id: usize,
    disabled: Signal<bool>,
    element: NodeRef<html::Button>,
}

// base ui's composite: one tab stop, arrows and home/end between the items
#[derive(Clone, Copy)]
pub(crate) struct Composite {
    orientation: Signal<&'static str>,
    loop_focus: Signal<bool>,
    home_end: bool,
    direction: Signal<&'static str>,
    highlighted: RwSignal<Option<usize>>,
    items: RwSignal<Vec<Item>>,
    next: StoredValue<usize>,
}

impl Composite {
    pub(crate) fn new(
        orientation: Signal<&'static str>,
        loop_focus: Signal<bool>,
        home_end: bool,
    ) -> Self {
        Self {
            orientation,
            loop_focus,
            home_end,
            direction: use_direction(),
            highlighted: RwSignal::new(None),
            items: RwSignal::new(Vec::new()),
            next: StoredValue::new(0),
        }
    }

    pub(crate) fn item(self, disabled: Signal<bool>, element: NodeRef<html::Button>) -> usize {
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
                element,
            })
        });
        on_cleanup(move || {
            self.items
                .try_update(|items| items.retain(|item| item.id != id));
        });
        id
    }

    // the highlighted item, or the first that isn't disabled
    fn current(&self) -> Option<usize> {
        self.items.with(|items| {
            let enabled = |id: usize| {
                items
                    .iter()
                    .any(|item| item.id == id && !item.disabled.get())
            };
            match self.highlighted.get() {
                Some(id) if enabled(id) => Some(id),
                _ => items
                    .iter()
                    .find(|item| !item.disabled.get())
                    .map(|item| item.id),
            }
        })
    }

    pub(crate) fn tabindex(self, id: usize) -> Signal<i32> {
        Signal::derive(move || if self.current() == Some(id) { 0 } else { -1 })
    }

    // base ui's composite buttons always say whether they're disabled
    pub(crate) fn item_attributes(
        self,
        id: usize,
        disabled: Signal<bool>,
    ) -> impl leptos::attr::Attribute {
        view! {
            <{..}
                aria-disabled=move || disabled.get().to_string()
                on:focus=move |_| self.highlighted.set(Some(id))
            />
        }
    }

    pub(crate) fn root_attributes(self) -> impl leptos::attr::Attribute {
        view! { <{..} on:keydown=move |event: ev::KeyboardEvent| self.navigate(&event) /> }
    }

    fn navigate(&self, event: &ev::KeyboardEvent) {
        let key = event.key();
        let modified = event.shift_key() || event.ctrl_key() || event.alt_key() || event.meta_key();
        if modified {
            return;
        }
        let rtl = self.direction.get_untracked() == "rtl";
        let orientation = self.orientation.get_untracked();
        let (forward_key, backward_key) = if rtl {
            ("ArrowLeft", "ArrowRight")
        } else {
            ("ArrowRight", "ArrowLeft")
        };
        let forward = (orientation != "vertical" && key == forward_key)
            || (orientation != "horizontal" && key == "ArrowDown");
        let backward = (orientation != "vertical" && key == backward_key)
            || (orientation != "horizontal" && key == "ArrowUp");
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
        elements.sort_by(|a, b| {
            if a.compare_document_position(b) & Node::DOCUMENT_POSITION_FOLLOWING != 0 {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });
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
                !element.has_attribute("disabled")
                    && element.get_attribute("aria-disabled").as_deref() != Some("true")
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
