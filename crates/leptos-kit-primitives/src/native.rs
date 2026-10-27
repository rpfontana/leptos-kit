use crate::{ChangeReason, Controlled, Orientation};
use leptos::prelude::*;
use leptos::tachys::html::element::ElementType;
use leptos::wasm_bindgen::JsCast;
use std::cmp::Ordering;
use web_sys::{HtmlDialogElement, HtmlElement, MouseEvent, Node};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Native {
    Modal,
    Dialog,
    Popover,
}

impl Native {
    pub(crate) fn is_open(self, element: &HtmlElement) -> bool {
        match self {
            Native::Modal | Native::Dialog => element.unchecked_ref::<HtmlDialogElement>().open(),
            Native::Popover => element.matches(":popover-open").unwrap_or(false),
        }
    }

    fn set_open(self, element: &HtmlElement, open: bool) {
        match self {
            Native::Modal | Native::Dialog => {
                let dialog = element.unchecked_ref::<HtmlDialogElement>();
                if !open {
                    dialog.close();
                } else if self == Native::Modal {
                    let _ = dialog.show_modal();
                } else {
                    dialog.show();
                }
            }
            Native::Popover => {
                let _ = if open {
                    element.show_popover()
                } else {
                    element.hide_popover()
                };
            }
        }
    }
}

pub(crate) struct NativeOpen<E: ElementType + 'static> {
    kind: Signal<Native>,
    element: NodeRef<E>,
    state: Controlled<bool>,
    reason: StoredValue<Option<ChangeReason>>,
    animated: Option<NodeRef<leptos::html::Div>>,
    generation: StoredValue<u64>,
}

impl<E: ElementType + 'static> Clone for NativeOpen<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E: ElementType + 'static> Copy for NativeOpen<E> {}

impl<E> NativeOpen<E>
where
    E: ElementType + 'static,
    E::Output: JsCast + Clone + AsRef<HtmlElement> + 'static,
{
    pub(crate) fn new(
        kind: impl Into<Signal<Native>>,
        element: NodeRef<E>,
        state: Controlled<bool>,
        animated: Option<NodeRef<leptos::html::Div>>,
    ) -> Self {
        let this = Self {
            kind: kind.into(),
            element,
            state,
            reason: StoredValue::new(None),
            animated,
            generation: StoredValue::new(0),
        };
        Effect::new(move || {
            let open = state.get();
            if let Some(element) = element.get() {
                this.put(element.as_ref(), open);
            }
        });
        this
    }

    pub(crate) fn state(&self) -> Controlled<bool> {
        self.state
    }

    pub(crate) fn expect(&self, reason: ChangeReason) {
        self.reason.set_value(Some(reason));
    }

    // call from both toggle and close, browsers disagree on which one ends a close
    pub(crate) fn sync(&self, fallback: ChangeReason) {
        let Some(Some(element)) = self.element.try_get_untracked() else {
            return;
        };
        let element = element.as_ref();
        let Some(kind) = self.kind.try_get_untracked() else {
            return;
        };
        let open = kind.is_open(element);
        let reason = self.reason.try_update_value(Option::take).flatten();
        self.state.set(open, reason.unwrap_or(fallback));
        self.put(element, self.state.get_untracked());
    }

    fn put(&self, element: &HtmlElement, open: bool) {
        let Some(kind) = self.kind.try_get_untracked() else {
            return;
        };
        let generation = self.generation.get_value() + 1;
        self.generation.set_value(generation);
        if kind.is_open(element) == open {
            return;
        }
        if open {
            kind.set_open(element, true);
            return;
        }

        let this = *self;
        let element = element.clone();
        let animated = match self.animated.and_then(|animated| animated.get_untracked()) {
            Some(animated) => animated.into(),
            None => element.clone(),
        };
        after_animations(animated, move || {
            if this.generation.try_get_value() == Some(generation) {
                kind.set_open(&element, false);
            }
        });
    }
}

// getAnimations is behind web_sys_unstable_apis
fn animations(element: &HtmlElement) -> web_sys::js_sys::Array {
    use web_sys::js_sys::{Array, Function, Reflect};
    Reflect::get(element, &"getAnimations".into())
        .ok()
        .and_then(|get| get.dyn_into::<Function>().ok())
        .and_then(|get| get.call0(element).ok())
        .and_then(|animations| animations.dyn_into::<Array>().ok())
        .unwrap_or_default()
}

// firefox styles the next frame without the one before, so a transition needs a frame painted in between
pub(crate) fn after_paint(done: impl FnOnce() + 'static) {
    request_animation_frame(move || request_animation_frame(done));
}

pub(crate) fn after_animations(element: HtmlElement, done: impl FnOnce() + 'static) {
    request_animation_frame(move || {
        let finished = web_sys::js_sys::Array::new();
        for animation in animations(&element).iter() {
            let animation: web_sys::Animation = animation.unchecked_into();
            if let Ok(promise) = animation.finished() {
                finished.push(&promise);
            }
        }
        if finished.length() == 0 {
            done();
            return;
        }
        let all = web_sys::js_sys::Promise::all_settled(&finished);
        leptos::task::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(all).await;
            done();
        });
    });
}

pub(crate) fn change<T: PartialEq + Send + Sync + 'static>(signal: RwSignal<T>, next: T) {
    signal.try_maybe_update(|value| {
        let changed = *value != next;
        if changed {
            *value = next;
        }
        (changed, ())
    });
}

pub(crate) fn along(element: &HtmlElement, axis: Orientation) -> f64 {
    f64::from(match axis {
        Orientation::Horizontal => element.offset_width(),
        Orientation::Vertical => element.offset_height(),
    })
}

pub(crate) fn client(event: &MouseEvent, axis: Orientation) -> f64 {
    f64::from(match axis {
        Orientation::Horizontal => event.client_x(),
        Orientation::Vertical => event.client_y(),
    })
}

pub(crate) fn observe_resize(element: &web_sys::Element, measure: impl Fn() + 'static) {
    let callback = leptos::wasm_bindgen::closure::Closure::<dyn Fn()>::new(measure);
    let Ok(observer) = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()) else {
        return;
    };
    observer.observe(element);
    let held = StoredValue::new_local((observer, callback));
    on_cleanup(move || {
        held.try_with_value(|(observer, _)| observer.disconnect());
    });
}

// measured before overflow: hidden takes the scrollbar away
pub(crate) fn keep_gutter() {
    let Some(html) = document().document_element() else {
        return;
    };
    let viewport = window()
        .inner_width()
        .ok()
        .and_then(|width| width.as_f64())
        .unwrap_or(0.0);
    if viewport > f64::from(html.client_width()) {
        let html: &web_sys::HtmlElement = html.unchecked_ref();
        let _ = html.style().set_property("scrollbar-gutter", "stable");
    }
}

pub(crate) fn release_gutter(own: Option<&Node>) {
    // another modal dialog or modal menu may still hold the page
    let held = document()
        .query_selector_all("dialog:modal, [data-modal-backdrop]:popover-open")
        .is_ok_and(|holders| {
            (0..holders.length())
                .filter_map(|index| holders.item(index))
                .any(|holder| !own.is_some_and(|own| own.is_same_node(Some(&holder))))
        });
    if held {
        return;
    }
    if let Some(html) = document().document_element() {
        let html: &web_sys::HtmlElement = html.unchecked_ref();
        let _ = html.style().remove_property("scrollbar-gutter");
    }
}

// sort_by wants a total order, so a node is equal to itself
pub(crate) fn document_order(a: &Node, b: &Node) -> Ordering {
    if a.is_same_node(Some(b)) {
        Ordering::Equal
    } else if a.compare_document_position(b) & Node::DOCUMENT_POSITION_FOLLOWING != 0 {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}
