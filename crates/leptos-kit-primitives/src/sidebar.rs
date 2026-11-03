use crate::{ChangeReason, Controlled, Keyword};
use leptos::context::Provider;
use leptos::ev;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;
use std::fmt;

const COOKIE: &str = "sidebar_state";
const COOKIE_MAX_AGE: u32 = 60 * 60 * 24 * 7;
const SHORTCUT: &str = "b";
const MOBILE: &str = "(max-width: 767px)";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Keyword)]
pub enum SidebarState {
    Expanded,
    Collapsed,
}

#[derive(Clone, Copy)]
pub struct SidebarContext {
    open: Controlled<bool>,
    open_mobile: RwSignal<bool>,
    is_mobile: RwSignal<bool>,
}

impl fmt::Debug for SidebarContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SidebarContext").finish_non_exhaustive()
    }
}

impl SidebarContext {
    pub fn state(&self) -> SidebarState {
        if self.open() {
            SidebarState::Expanded
        } else {
            SidebarState::Collapsed
        }
    }

    pub fn open(&self) -> bool {
        self.open.get()
    }

    pub fn set_open(&self, open: bool) {
        if let Ok(document) = document().dyn_into::<web_sys::HtmlDocument>() {
            let cookie = format!("{COOKIE}={open}; path=/; max-age={COOKIE_MAX_AGE}");
            let _ = document.set_cookie(&cookie);
        }
        self.open.set(open, ChangeReason::None);
    }

    pub fn open_mobile(&self) -> bool {
        self.open_mobile.get()
    }

    pub fn set_open_mobile(&self, open: bool) {
        self.open_mobile.set(open);
    }

    pub fn is_mobile(&self) -> bool {
        self.is_mobile.get()
    }

    pub fn toggle_sidebar(&self) {
        if self.is_mobile.get_untracked() {
            self.open_mobile.update(|open| *open = !*open);
        } else {
            self.set_open(!self.open.get_untracked());
        }
    }
}

pub fn use_sidebar() -> SidebarContext {
    use_context().expect("use_sidebar must be used within a SidebarProvider")
}

#[leptos::component]
pub fn SidebarRoot(
    #[prop(default = true)] default_open: bool,
    #[prop(optional_no_strip, into)] open: Option<Signal<bool>>,
    #[prop(optional_no_strip, into)] on_open_change: Option<Callback<bool>>,
    children: Children,
) -> impl IntoView {
    let on_open_change = on_open_change
        .map(|callback| Callback::new(move |(open, _): (bool, ChangeReason)| callback.run(open)));
    let context = SidebarContext {
        open: Controlled::new(open, default_open, on_open_change),
        open_mobile: RwSignal::new(false),
        is_mobile: RwSignal::new(false),
    };
    // the server and the hydration both render the desktop sidebar
    Effect::new(move |_| {
        let Ok(Some(query)) = window().match_media(MOBILE) else {
            return;
        };
        context.is_mobile.set(query.matches());
        let changed = query.clone();
        let listener = Closure::<dyn Fn()>::new(move || context.is_mobile.set(changed.matches()));
        query.set_onchange(Some(listener.as_ref().unchecked_ref()));
        let held = StoredValue::new_local((query, listener));
        on_cleanup(move || {
            held.try_with_value(|(query, _)| query.set_onchange(None));
        });
    });
    Effect::new(move |_| {
        let shortcut = window_event_listener(ev::keydown, move |event| {
            if event.key() == SHORTCUT && (event.meta_key() || event.ctrl_key()) {
                event.prevent_default();
                context.toggle_sidebar();
            }
        });
        on_cleanup(move || shortcut.remove());
    });
    view! { <Provider value=context>{children()}</Provider> }
}
