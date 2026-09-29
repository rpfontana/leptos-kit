use crate::Controlled;
use leptos::prelude::*;
use leptos::tachys::html::element::ElementType;
use leptos::wasm_bindgen::JsCast;
use web_sys::{HtmlDetailsElement, HtmlDialogElement, HtmlElement};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Native {
    Modal,
    Dialog,
    Popover,
    Details,
}

impl Native {
    pub fn is_open(self, element: &HtmlElement) -> bool {
        match self {
            Native::Modal | Native::Dialog => element.unchecked_ref::<HtmlDialogElement>().open(),
            Native::Popover => element.matches(":popover-open").unwrap_or(false),
            Native::Details => element.unchecked_ref::<HtmlDetailsElement>().open(),
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
            Native::Details => element.unchecked_ref::<HtmlDetailsElement>().set_open(open),
        }
    }
}

pub struct NativeOpen<E: ElementType + 'static> {
    kind: Native,
    element: NodeRef<E>,
    state: Controlled<bool>,
    reason: StoredValue<Option<&'static str>>,
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
    pub fn new(
        kind: Native,
        element: NodeRef<E>,
        state: Controlled<bool>,
        animated: Option<NodeRef<leptos::html::Div>>,
    ) -> Self {
        let this = Self {
            kind,
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

    pub fn state(&self) -> Controlled<bool> {
        self.state
    }

    pub fn expect(&self, reason: &'static str) {
        self.reason.set_value(Some(reason));
    }

    // call from both toggle and close, browsers disagree on which one ends a close
    pub fn sync(&self, fallback: &'static str) {
        let Some(element) = self.element.get_untracked() else {
            return;
        };
        let element = element.as_ref();
        let open = self.kind.is_open(element);
        let reason = self.reason.try_update_value(Option::take).flatten();
        self.state.set(open, reason.unwrap_or(fallback));
        self.put(element, self.state.get_untracked());
    }

    fn put(&self, element: &HtmlElement, open: bool) {
        let generation = self.generation.get_value() + 1;
        self.generation.set_value(generation);
        if self.kind.is_open(element) == open {
            return;
        }
        if open {
            self.kind.set_open(element, true);
            return;
        }

        let this = *self;
        let element = element.clone();
        let animated = match self.animated.and_then(|animated| animated.get_untracked()) {
            Some(animated) => animated.into(),
            None => element.clone(),
        };
        after_animations(animated, move || {
            if this.generation.get_value() == generation {
                this.kind.set_open(&element, false);
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

fn after_animations(element: HtmlElement, done: impl FnOnce() + 'static) {
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
