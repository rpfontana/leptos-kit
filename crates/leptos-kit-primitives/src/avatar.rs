use leptos::context::Provider;
use leptos::html;
use leptos::prelude::*;
use leptos_kit_macros::component;
use std::time::Duration;

#[derive(Clone, Copy)]
struct AvatarContext {
    status: RwSignal<&'static str>,
}

#[component(element = "span")]
pub fn AvatarRoot(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = AvatarContext {
        status: RwSignal::new("idle"),
    };
    view! {
        <span {..attrs}>
            <Provider value=context>{children.map(|children| children())}</Provider>
        </span>
    }
}

// hidden until it loads rather than mounted then, so the browser fetches it before the wasm
#[component(element = "img")]
pub fn AvatarImage(
    #[prop(optional_no_strip, into)] src: Option<Signal<String>>,
    #[prop(optional, into)] keep_mounted: Signal<bool>,
    #[prop(optional_no_strip, into)] on_loading_status_change: Option<Callback<&'static str>>,
) -> impl IntoView {
    let context = expect_context::<AvatarContext>();
    let image = NodeRef::<html::Img>::new();
    let status = RwSignal::new("idle");
    let report = move |next: &'static str| {
        if status
            .try_get_untracked()
            .is_none_or(|status| status == next)
        {
            return;
        }
        status.set(next);
        context.status.set(next);
        if let Some(callback) = on_loading_status_change {
            callback.run(next);
        }
    };
    // also catches an image that loaded before the wasm, its load event went to nobody
    Effect::new(move || {
        if let Some(src) = src {
            src.track();
        }
        request_animation_frame(move || {
            let Some(Some(image)) = image.try_get_untracked() else {
                return;
            };
            report(match (image.complete(), image.natural_width() > 0) {
                (false, _) => "loading",
                (true, true) => "loaded",
                (true, false) => "error",
            });
        });
    });
    on_cleanup(move || {
        context.status.try_set("idle");
    });
    let shown = move || status.get() == "loaded";

    view! {
        <img
            {..attrs}
            node_ref=image
            src=move || src.map(|src| src.get())
            hidden=move || !keep_mounted.get() && !shown()
            aria-hidden=move || (keep_mounted.get() && !shown()).then_some("true")
            data-loading=move || (keep_mounted.get() && status.get() == "loading").then_some("")
            data-error=move || (keep_mounted.get() && status.get() == "error").then_some("")
            on:load=move |_| report("loaded")
            on:error=move |_| report("error")
        />
    }
}

#[component(element = "span")]
pub fn AvatarFallback(
    #[prop(optional, into)] delay: Signal<u32>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<AvatarContext>();
    let waited = RwSignal::new(false);
    Effect::new(move || {
        let delay = delay.get();
        if delay == 0 {
            return;
        }
        let timeout = set_timeout_with_handle(
            move || {
                waited.try_set(true);
            },
            Duration::from_millis(delay.into()),
        )
        .ok();
        on_cleanup(move || {
            if let Some(timeout) = timeout {
                timeout.clear();
            }
        });
    });
    let shown = move || context.status.get() != "loaded" && (delay.get() == 0 || waited.get());
    view! {
        <span {..attrs} hidden=move || !shown()>
            {children.map(|children| children())}
        </span>
    }
}
