use leptos::prelude::*;
use leptos::tachys::html::element::ElementType;
use leptos::wasm_bindgen::JsCast;
use web_sys::HtmlElement;

/// the same id on the server and the hydrated client, though islands can hydrate out of order
pub fn use_id() -> String {
    #[cfg(any(feature = "ssr", feature = "hydrate"))]
    if let Some(shared) = Owner::current_shared_context() {
        return format!("lk-{}", shared.next_id().into_inner());
    }

    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    // its own prefix, in case a hydrated page also mounts a client-only root
    format!("lk-c{}", NEXT.fetch_add(1, Ordering::Relaxed))
}

// once the element is in the dom its own id wins, the client's count can drift from the server's
pub(crate) struct ElementId<E: ElementType + 'static> {
    element: NodeRef<E>,
    fallback: StoredValue<String>,
}

impl<E: ElementType + 'static> Clone for ElementId<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E: ElementType + 'static> Copy for ElementId<E> {}

impl<E: ElementType + 'static> PartialEq for ElementId<E> {
    fn eq(&self, other: &Self) -> bool {
        self.fallback == other.fallback
    }
}

impl<E> ElementId<E>
where
    E: ElementType + 'static,
    E::Output: JsCast + Clone + AsRef<HtmlElement> + 'static,
{
    pub(crate) fn new(element: NodeRef<E>, fallback: String) -> Self {
        Self {
            element,
            fallback: StoredValue::new(fallback),
        }
    }

    pub(crate) fn get(&self) -> String {
        self.element
            .try_get_untracked()
            .flatten()
            .map(|element| AsRef::<HtmlElement>::as_ref(&element).id())
            .filter(|id| !id.is_empty())
            .or_else(|| self.fallback.try_get_value())
            .unwrap_or_default()
    }
}
