/// the same id on the server and the hydrated client
pub fn use_id() -> String {
    #[cfg(any(feature = "ssr", feature = "hydrate"))]
    if let Some(shared) = leptos::prelude::Owner::current_shared_context() {
        return format!("lk-{}", shared.next_id().into_inner());
    }

    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    // its own prefix, in case a hydrated page also mounts a client-only root
    format!("lk-c{}", NEXT.fetch_add(1, Ordering::Relaxed))
}
