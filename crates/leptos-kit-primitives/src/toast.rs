use crate::button::button_attributes;
use crate::id::ElementId;
use crate::native::{after_animations, observe_resize};
use crate::swipe::{REVERSE_CANCEL_THRESHOLD, SwipeDirection, displacement, transform_of};
use crate::{Keyword, Orientation, use_id};
use leptos::context::Provider;
use leptos::either::Either;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::tachys::html::event::on;
use leptos::wasm_bindgen::JsCast;
use leptos_kit_macros::component;
use send_wrapper::SendWrapper;
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use web_sys::{Element, HtmlElement, Node};

const SWIPE_THRESHOLD: f64 = 40.0;
const OPPOSITE_DIRECTION_DAMPING: f64 = 0.5;
const SWIPE_IGNORE: &str =
    "button,a,input,textarea,[role=\"button\"],[data-base-ui-swipe-ignore],[data-swipe-ignore]";
const VISUALLY_HIDDEN: &str = "clip-path: inset(50%); overflow: hidden; white-space: nowrap; border: 0px; \
     padding: 0px; width: 1px; height: 1px; margin: -1px; position: fixed; top: 0px; left: 0px;";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Keyword)]
pub enum ToastPriority {
    #[default]
    Low,
    High,
}

#[derive(Clone, Debug)]
pub struct ToastActionOptions {
    pub label: String,
    // gets the toast's id, which `add` only returns afterwards
    pub on_click: Option<Callback<String>>,
}

// in an update, a `None` keeps what the toast had
#[derive(Clone, Debug, Default)]
pub struct ToastOptions {
    pub id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub r#type: Option<String>,
    pub timeout: Option<u32>,
    pub priority: Option<ToastPriority>,
    pub action: Option<ToastActionOptions>,
    pub on_close: Option<Callback<()>>,
    pub on_remove: Option<Callback<()>>,
}

impl From<&str> for ToastOptions {
    fn from(description: &str) -> Self {
        Self {
            description: Some(description.to_string()),
            ..Self::default()
        }
    }
}

impl From<String> for ToastOptions {
    fn from(description: String) -> Self {
        Self {
            description: Some(description),
            ..Self::default()
        }
    }
}

enum Event {
    Add(ToastOptions),
    Close(Option<String>),
    Update(String, ToastOptions),
}

type Listener = Arc<dyn Fn(&Event) + Send + Sync>;

#[derive(Default)]
struct Listeners {
    next: AtomicUsize,
    all: Mutex<Vec<(usize, Listener)>>,
}

#[derive(Clone, Default)]
pub struct ToastManager(Arc<Listeners>);

impl fmt::Debug for ToastManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToastManager").finish_non_exhaustive()
    }
}

impl ToastManager {
    pub fn new() -> Self {
        Self::default()
    }

    fn emit(&self, event: &Event) {
        let listeners: Vec<Listener> = self
            .0
            .all
            .lock()
            .expect("toast listeners lock poisoned")
            .iter()
            .map(|(_, listener)| Arc::clone(listener))
            .collect();
        for listener in listeners {
            listener(event);
        }
    }

    fn subscribe(&self, listener: Listener) -> usize {
        let key = self.0.next.fetch_add(1, Ordering::Relaxed);
        self.0
            .all
            .lock()
            .expect("toast listeners lock poisoned")
            .push((key, listener));
        key
    }

    fn unsubscribe(&self, key: usize) {
        self.0
            .all
            .lock()
            .expect("toast listeners lock poisoned")
            .retain(|(own, _)| *own != key);
    }

    pub fn add(&self, options: impl Into<ToastOptions>) -> String {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let mut options = options.into();
        let id = options
            .id
            .clone()
            .unwrap_or_else(|| format!("toast-{}", NEXT.fetch_add(1, Ordering::Relaxed)));
        options.id = Some(id.clone());
        self.emit(&Event::Add(options));
        id
    }

    pub fn close(&self, id: &str) {
        self.emit(&Event::Close(Some(id.to_string())));
    }

    pub fn close_all(&self) {
        self.emit(&Event::Close(None));
    }

    pub fn update(&self, id: &str, options: impl Into<ToastOptions>) {
        self.emit(&Event::Update(id.to_string(), options.into()));
    }

    pub fn promise<T: 'static, E: 'static, S: Into<ToastOptions>, F: Into<ToastOptions>>(
        &self,
        future: impl Future<Output = Result<T, E>> + 'static,
        loading: impl Into<ToastOptions>,
        success: impl FnOnce(T) -> S + 'static,
        error: impl FnOnce(E) -> F + 'static,
    ) -> String {
        let mut loading = loading.into();
        loading.r#type = Some(String::from("loading"));
        let id = self.add(loading);
        let manager = self.clone();
        let settled = id.clone();
        leptos::task::spawn_local(async move {
            let (mut options, kind): (ToastOptions, _) = match future.await {
                Ok(value) => (success(value).into(), "success"),
                Err(cause) => (error(cause).into(), "error"),
            };
            options.r#type = Some(kind.to_string());
            manager.update(&settled, options);
        });
        id
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Starting,
    Ending,
}

#[derive(Clone)]
struct Stored {
    id: String,
    title: Option<String>,
    description: Option<String>,
    kind: Option<String>,
    timeout: Option<u32>,
    priority: ToastPriority,
    action: Option<ToastActionOptions>,
    on_close: Option<Callback<()>>,
    on_remove: Option<Callback<()>>,
    phase: Option<Phase>,
    height: Option<f64>,
}

impl Stored {
    fn new(id: String, options: ToastOptions) -> Self {
        Self {
            id,
            title: options.title,
            description: options.description,
            kind: options.r#type,
            timeout: options.timeout,
            priority: options.priority.unwrap_or_default(),
            action: options.action,
            on_close: options.on_close,
            on_remove: options.on_remove,
            phase: Some(Phase::Starting),
            height: None,
        }
    }

    fn merge(&mut self, options: ToastOptions) {
        let ToastOptions {
            id: _,
            title,
            description,
            r#type,
            timeout,
            priority,
            action,
            on_close,
            on_remove,
        } = options;
        self.title = title.or(self.title.take());
        self.description = description.or(self.description.take());
        self.kind = r#type.or(self.kind.take());
        self.timeout = timeout.or(self.timeout);
        self.priority = priority.unwrap_or(self.priority);
        self.action = action.or(self.action.take());
        self.on_close = on_close.or(self.on_close);
        self.on_remove = on_remove.or(self.on_remove);
    }

    fn loading(&self) -> bool {
        self.kind.as_deref() == Some("loading")
    }

    fn ending(&self) -> bool {
        self.phase == Some(Phase::Ending)
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
struct Place {
    index: usize,
    visible: Option<usize>,
    offset: f64,
    limited: bool,
}

fn place(toasts: &[Stored], id: &str, limit: usize) -> Option<Place> {
    let (mut visible, mut offset) = (0, 0.0);
    for (index, toast) in toasts.iter().enumerate() {
        let ending = toast.ending();
        if toast.id == id {
            return Some(Place {
                index,
                visible: (!ending).then_some(visible),
                offset,
                limited: !ending && visible >= limit,
            });
        }
        offset += toast.height.unwrap_or_default();
        if !ending {
            visible += 1;
        }
    }
    None
}

struct Timer {
    handle: Option<TimeoutHandle>,
    start: f64,
    delay: f64,
    remaining: f64,
}

#[derive(Clone, Copy)]
struct Store {
    manager: StoredValue<ToastManager>,
    toasts: RwSignal<Vec<Stored>>,
    hovering: RwSignal<bool>,
    focused: RwSignal<bool>,
    window_focused: RwSignal<bool>,
    timeout: Signal<u32>,
    limit: Signal<usize>,
    viewport: NodeRef<html::Div>,
    // not thread-local: the server may drop them on another thread, which panics
    prev_focus: StoredValue<Option<SendWrapper<HtmlElement>>>,
    timers: StoredValue<HashMap<String, Timer>>,
    paused: StoredValue<bool>,
    roots: StoredValue<HashMap<String, NodeRef<html::Div>>>,
    // the pointer left while toasts were closing: the stack collapses once they're gone
    leaving: StoredValue<bool>,
    touching: StoredValue<bool>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct ToastContext(StoredValue<Store>);

impl ToastContext {
    fn store(self) -> Store {
        self.0.get_value()
    }

    fn try_store(self) -> Option<Store> {
        self.0.try_get_value()
    }

    fn expanded(self) -> bool {
        self.try_store()
            .is_some_and(|store| store.hovering.get() || store.focused.get())
    }

    fn away(self) -> bool {
        self.try_store().is_some_and(|store| {
            store.hovering.get_untracked()
                || store.focused.get_untracked()
                || !store.window_focused.get_untracked()
        })
    }

    fn with_toast<T>(self, id: &str, read: impl FnOnce(&Stored) -> T) -> Option<T> {
        self.try_store()?
            .toasts
            .with(|toasts| toasts.iter().find(|toast| toast.id == id).map(read))
    }

    fn place(self, id: &str) -> Option<Place> {
        let store = self.try_store()?;
        let limit = store.limit.get();
        store.toasts.with(|toasts| place(toasts, id, limit))
    }

    fn handle(self, event: &Event) {
        match event {
            Event::Add(options) => self.add(options.clone()),
            Event::Close(id) => self.close(id.as_deref()),
            Event::Update(id, options) => self.update(id, options.clone(), false),
        }
    }

    fn set_toasts(self, toasts: Vec<Stored>, clear: Option<bool>) {
        let Some(store) = self.try_store() else {
            return;
        };
        let clear = clear.unwrap_or(toasts.is_empty());
        store.toasts.set(toasts);
        if clear {
            store.hovering.set(false);
            store.focused.set(false);
        }
    }

    fn add(self, options: ToastOptions) {
        let Some(store) = self.try_store() else {
            return;
        };
        let id = options.id.clone().unwrap_or_default();
        let existing = store.toasts.with_untracked(|toasts| {
            toasts
                .iter()
                .find(|toast| toast.id == id)
                .map(Stored::ending)
        });
        match existing {
            Some(true) => self.remove(&id, true),
            Some(false) => {
                self.update(&id, options, true);
                return;
            }
            None => {}
        }
        let toast = Stored::new(id.clone(), options);
        let duration = toast
            .timeout
            .unwrap_or_else(|| store.timeout.get_untracked());
        let loading = toast.loading();
        let mut toasts = vec![toast];
        toasts.extend(store.toasts.get_untracked());
        self.set_toasts(toasts, None);
        if !loading && duration > 0 {
            self.schedule(id, duration);
        }
        if self.away() {
            self.pause();
        }
    }

    fn update(self, id: &str, options: ToastOptions, reset: bool) {
        let Some(store) = self.try_store() else {
            return;
        };
        let Some(previous) = store
            .toasts
            .with_untracked(|toasts| toasts.iter().find(|toast| toast.id == id).cloned())
        else {
            return;
        };
        // an ending toast takes no more updates, so a late promise can't bring it back
        if previous.ending() {
            return;
        }
        let timeout_given = options.timeout.is_some();
        let mut next = previous.clone();
        next.merge(options);
        let default = store.timeout.get_untracked();
        let (was, now) = (
            previous.timeout.unwrap_or(default),
            next.timeout.unwrap_or(default),
        );
        let should = !next.loading() && now > 0;
        let toasts = store.toasts.with_untracked(|toasts| {
            toasts
                .iter()
                .map(|toast| {
                    if toast.id == id {
                        next.clone()
                    } else {
                        toast.clone()
                    }
                })
                .collect()
        });
        self.set_toasts(toasts, Some(false));
        let has = store.timers.with_value(|timers| timers.contains_key(id));
        if !should && has {
            self.clear_timer(id);
            return;
        }
        if should && (!has || was != now || timeout_given || previous.loading() || reset) {
            self.clear_timer(id);
            self.schedule(id.to_string(), now);
            if self.away() {
                self.pause();
            }
        }
    }

    fn measured(self, id: &str, height: f64) {
        let Some(store) = self.try_store() else {
            return;
        };
        store.toasts.try_maybe_update(|toasts| {
            let Some(toast) = toasts
                .iter_mut()
                .find(|toast| toast.id == id && !toast.ending())
            else {
                return (false, ());
            };
            let changed = toast.height != Some(height) || toast.phase.is_some();
            toast.height = Some(height);
            toast.phase = None;
            (changed, ())
        });
    }

    fn close(self, id: Option<&str>) {
        let Some(store) = self.try_store() else {
            return;
        };
        let toasts = store.toasts.get_untracked();
        let closing: Vec<Stored> = match id {
            None => {
                self.clear_timers();
                toasts.clone()
            }
            Some(id) => {
                let Some(toast) = toasts.iter().find(|toast| toast.id == id) else {
                    return;
                };
                self.clear_timer(id);
                vec![toast.clone()]
            }
        };
        let next: Vec<Stored> = toasts
            .into_iter()
            .map(|mut toast| {
                if id.is_none_or(|id| id == toast.id) {
                    toast.phase = Some(Phase::Ending);
                    toast.height = None;
                }
                toast
            })
            .collect();
        let none_left = next.iter().all(Stored::ending);
        self.set_toasts(next, Some(none_left));
        for toast in closing.iter().filter(|toast| !toast.ending()) {
            if let Some(on_close) = toast.on_close {
                on_close.run(());
            }
        }
        self.move_focus(id);
    }

    fn remove(self, id: &str, skip_on_remove: bool) {
        let Some(store) = self.try_store() else {
            return;
        };
        let mut toasts = store.toasts.get_untracked();
        let Some(index) = toasts.iter().position(|toast| toast.id == id) else {
            return;
        };
        let toast = toasts.remove(index);
        if !skip_on_remove && let Some(on_remove) = toast.on_remove {
            on_remove.run(());
        }
        self.set_toasts(toasts, None);
        self.flush_leave();
    }

    fn move_focus(self, id: Option<&str>) {
        let Some(store) = self.try_store() else {
            return;
        };
        let Some(viewport) = store.viewport.get_untracked() else {
            return;
        };
        let Some(active) = document().active_element() else {
            return;
        };
        if !viewport.contains(Some(&active)) || !active.matches(":focus-visible").unwrap_or(false) {
            return;
        }
        let Some(id) = id else {
            self.restore_focus();
            return;
        };
        let toasts = store.toasts.get_untracked();
        let Some(at) = toasts.iter().position(|toast| toast.id == id) else {
            return;
        };
        let next = toasts[at + 1..]
            .iter()
            .chain(toasts[..at].iter().rev())
            .find(|toast| !toast.ending())
            .and_then(|toast| {
                store
                    .roots
                    .with_value(|roots| roots.get(&toast.id).copied())
            })
            .and_then(|root| root.get_untracked());
        match next {
            Some(root) => {
                let _ = root.focus();
            }
            None => self.restore_focus(),
        }
    }

    fn restore_focus(self) {
        if let Some(Some(element)) = self
            .try_store()
            .and_then(|store| store.prev_focus.try_get_value())
        {
            let options = web_sys::FocusOptions::new();
            options.set_prevent_scroll(true);
            let _ = element.focus_with_options(&options);
        }
    }

    fn schedule(self, id: String, delay: u32) {
        let Some(store) = self.try_store() else {
            return;
        };
        let delay = f64::from(delay);
        let handle = (!self.away())
            .then(|| self.start_timer(id.clone(), delay))
            .flatten();
        store.timers.update_value(|timers| {
            timers.insert(
                id,
                Timer {
                    handle,
                    start: now(),
                    delay,
                    remaining: delay,
                },
            );
        });
    }

    fn start_timer(self, id: String, delay: f64) -> Option<TimeoutHandle> {
        set_timeout_with_handle(
            move || {
                if let Some(store) = self.try_store() {
                    store.timers.update_value(|timers| {
                        timers.remove(&id);
                    });
                    self.reset_paused();
                    self.close(Some(&id));
                }
            },
            Duration::from_secs_f64(delay.max(0.0) / 1000.0),
        )
        .ok()
    }

    fn clear_timer(self, id: &str) {
        let Some(store) = self.try_store() else {
            return;
        };
        store.timers.update_value(|timers| {
            if let Some(timer) = timers.remove(id)
                && let Some(handle) = timer.handle
            {
                handle.clear();
            }
        });
        self.reset_paused();
    }

    fn clear_timers(self) {
        let Some(store) = self.try_store() else {
            return;
        };
        store.timers.update_value(|timers| {
            for (_, timer) in timers.drain() {
                if let Some(handle) = timer.handle {
                    handle.clear();
                }
            }
        });
        store.paused.set_value(false);
    }

    fn reset_paused(self) {
        if let Some(store) = self.try_store()
            && store.timers.with_value(HashMap::is_empty)
        {
            store.paused.set_value(false);
        }
    }

    fn pause(self) {
        let Some(store) = self.try_store() else {
            return;
        };
        if store.paused.get_value() {
            return;
        }
        store.paused.set_value(true);
        let at = now();
        store.timers.update_value(|timers| {
            for timer in timers.values_mut() {
                if let Some(handle) = timer.handle.take() {
                    handle.clear();
                    timer.remaining = (timer.remaining - (at - timer.start)).max(0.0);
                }
            }
        });
    }

    fn resume(self) {
        let Some(store) = self.try_store() else {
            return;
        };
        if !store.paused.get_value() {
            return;
        }
        store.paused.set_value(false);
        let at = now();
        let ids: Vec<(String, f64)> = store.timers.with_value(|timers| {
            timers
                .iter()
                .map(|(id, timer)| {
                    let remaining = if timer.remaining > 0.0 {
                        timer.remaining
                    } else {
                        timer.delay
                    };
                    (id.clone(), remaining)
                })
                .collect()
        });
        for (id, remaining) in ids {
            let handle = self.start_timer(id.clone(), remaining);
            store.timers.update_value(|timers| {
                if let Some(timer) = timers.get_mut(&id) {
                    if let Some(old) = timer.handle.take() {
                        old.clear();
                    }
                    timer.handle = handle;
                    timer.remaining = remaining;
                    timer.start = at;
                }
            });
        }
    }

    fn resume_if_window_focused(self) {
        if self
            .try_store()
            .is_some_and(|store| store.window_focused.get_untracked())
        {
            self.resume();
        }
    }

    fn flush_leave(self) {
        let Some(store) = self.try_store() else {
            return;
        };
        let ending = store
            .toasts
            .with_untracked(|toasts| toasts.iter().any(Stored::ending));
        if ending || store.touching.get_value() || !store.leaving.get_value() {
            return;
        }
        self.resume_if_window_focused();
        store.hovering.set(false);
        store.leaving.set_value(false);
    }
}

fn now() -> f64 {
    web_sys::js_sys::Date::now()
}

fn focus_visible(element: &Element) -> bool {
    element.matches(":focus-visible").unwrap_or(false)
}

#[derive(Clone)]
pub struct ToastObject {
    id: Arc<str>,
    context: ToastContext,
}

impl fmt::Debug for ToastObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToastObject")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl PartialEq for ToastObject {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ToastObject {}

impl ToastObject {
    pub fn id(&self) -> &str {
        &self.id
    }

    fn read<T>(&self, read: impl FnOnce(&Stored) -> T) -> Option<T> {
        self.context.with_toast(&self.id, read)
    }

    pub fn title(&self) -> Option<String> {
        self.read(|toast| toast.title.clone()).flatten()
    }

    pub fn description(&self) -> Option<String> {
        self.read(|toast| toast.description.clone()).flatten()
    }

    pub fn r#type(&self) -> Option<String> {
        self.read(|toast| toast.kind.clone()).flatten()
    }

    pub fn priority(&self) -> ToastPriority {
        self.read(|toast| toast.priority).unwrap_or_default()
    }
}

#[derive(Clone, Debug)]
pub struct UseToastManager {
    pub toasts: Signal<Vec<ToastObject>>,
    pub manager: ToastManager,
}

impl UseToastManager {
    pub fn add(&self, options: impl Into<ToastOptions>) -> String {
        self.manager.add(options)
    }

    pub fn close(&self, id: &str) {
        self.manager.close(id);
    }

    pub fn close_all(&self) {
        self.manager.close_all();
    }

    pub fn update(&self, id: &str, options: impl Into<ToastOptions>) {
        self.manager.update(id, options);
    }
}

pub fn use_toast_manager() -> UseToastManager {
    let context = expect_context::<ToastContext>();
    let store = context.store();
    let toasts = Signal::derive(move || {
        store.toasts.with(|toasts| {
            toasts
                .iter()
                .map(|toast| ToastObject {
                    id: Arc::from(toast.id.as_str()),
                    context,
                })
                .collect()
        })
    });
    UseToastManager {
        toasts,
        manager: store.manager.get_value(),
    }
}

#[leptos::component]
pub fn ToastProvider(
    #[prop(into, default = 5000.into())] timeout: Signal<u32>,
    #[prop(into, default = 3.into())] limit: Signal<usize>,
    #[prop(optional)] toast_manager: Option<ToastManager>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let manager = toast_manager.unwrap_or_default();
    let context = ToastContext(StoredValue::new(Store {
        manager: StoredValue::new(manager.clone()),
        toasts: RwSignal::new(Vec::new()),
        hovering: RwSignal::new(false),
        focused: RwSignal::new(false),
        window_focused: RwSignal::new(true),
        timeout,
        limit,
        viewport: NodeRef::new(),
        prev_focus: StoredValue::new(None),
        timers: StoredValue::new(HashMap::new()),
        paused: StoredValue::new(false),
        roots: StoredValue::new(HashMap::new()),
        leaving: StoredValue::new(false),
        touching: StoredValue::new(false),
    }));
    let store = context.store();
    // toasts only ever come from the page's script, so the manager is listened to there
    Effect::new(move |_| {
        let manager = manager.clone();
        let key = manager.subscribe(Arc::new(move |event| context.handle(event)));
        on_cleanup(move || manager.unsubscribe(key));
    });
    on_cleanup(move || context.clear_timers());
    let alerts = move || {
        if store.focused.get() {
            return Vec::new();
        }
        store.toasts.with(|toasts| {
            toasts
                .iter()
                .filter(|toast| toast.priority == ToastPriority::High)
                .map(|toast| (toast.title.clone(), toast.description.clone()))
                .collect::<Vec<_>>()
        })
    };
    view! {
        <Provider value=context>
            {children.map(|children| children())}
            {move || {
                let alerts = alerts();
                (!alerts.is_empty())
                    .then(|| {
                        view! {
                            <div style=VISUALLY_HIDDEN>
                                {alerts
                                    .into_iter()
                                    .map(|(title, description)| {
                                        view! {
                                            <div role="alert" aria-atomic="true">
                                                <div>{title}</div>
                                                <div>{description}</div>
                                            </div>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        }
                    })
            }}
        </Provider>
    }
}

#[leptos::component]
pub fn ToastPortal(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    children.map(|children| children())
}

#[component(element = "div", skip(tabindex, role))]
pub fn ToastViewport(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<ToastContext>();
    let store = context.store();
    let empty = move || store.toasts.with(Vec::is_empty);
    let guarding = StoredValue::new(false);
    Effect::new(move |_| {
        if empty() {
            return;
        }
        let keys = window_event_listener(ev::keydown, move |event| {
            let Some(viewport) = store.viewport.get_untracked() else {
                return;
            };
            let on_viewport = event
                .target()
                .and_then(|target| target.dyn_into::<Node>().ok())
                .is_some_and(|target| viewport.is_same_node(Some(&target)));
            if event.key() == "F6" && !on_viewport {
                event.prevent_default();
                store.prev_focus.set_value(
                    document()
                        .active_element()
                        .and_then(|active| active.dyn_into::<HtmlElement>().ok())
                        .map(SendWrapper::new),
                );
                let options = web_sys::FocusOptions::new();
                options.set_prevent_scroll(true);
                let _ = viewport.focus_with_options(&options);
                context.pause();
                store.focused.set(true);
            }
        });
        let blur = window_event_listener(ev::blur, move |event| {
            let on_window = event
                .target()
                .is_some_and(|target| target.dyn_ref::<web_sys::Window>().is_some());
            if on_window {
                store.window_focused.set(false);
                context.pause();
            }
        });
        let focus = window_event_listener(ev::focus, move |event| {
            if event.related_target().is_some() {
                return;
            }
            let target = event.target();
            let on_window = target
                .as_ref()
                .is_some_and(|target| target.dyn_ref::<web_sys::Window>().is_some());
            let inside = store.viewport.get_untracked().is_some_and(|viewport| {
                target
                    .and_then(|target| target.dyn_into::<Node>().ok())
                    .is_some_and(|target| viewport.contains(Some(&target)))
            });
            let visible = document()
                .active_element()
                .is_some_and(|active| focus_visible(&active));
            if on_window || !inside || !visible {
                context.resume();
            }
            set_timeout(move || store.window_focused.set(true), Duration::ZERO);
        });
        let press = window_event_listener(ev::pointerdown, move |event| {
            if event.pointer_type() != "touch" {
                return;
            }
            let inside = store.viewport.get_untracked().is_some_and(|viewport| {
                event
                    .target()
                    .and_then(|target| target.dyn_into::<Node>().ok())
                    .is_some_and(|target| viewport.contains(Some(&target)))
            });
            if !inside {
                context.resume();
                store.hovering.set(false);
                store.focused.set(false);
            }
        });
        on_cleanup(move || {
            keys.remove();
            blur.remove();
            focus.remove();
            press.remove();
        });
    });
    let hover = move || {
        context.pause();
        store.hovering.set(true);
        store.leaving.set_value(false);
    };
    let focus_in = move || {
        if guarding.get_value() {
            guarding.set_value(false);
            return;
        }
        if store.focused.get_untracked() {
            return;
        }
        // a click inside isn't keyboard navigation, it doesn't keep the stack open
        if document()
            .active_element()
            .is_some_and(|active| focus_visible(&active))
        {
            store.focused.set(true);
            context.pause();
        }
    };
    let frontmost = move || {
        store.toasts.with(|toasts| {
            toasts
                .first()
                .and_then(|toast| toast.height)
                .map(|height| format!("{height}px"))
        })
    };
    view! {
        <div
            {..attrs}
            node_ref=store.viewport
            tabindex="-1"
            role="region"
            aria-live="polite"
            aria-atomic="false"
            aria-relevant="additions text"
            aria-label="Notifications"
            data-expanded=move || context.expanded().then_some("")
            style:--toast-frontmost-height=frontmost
            on:mouseenter=move |_| hover()
            on:mousemove=move |_| hover()
            on:mouseleave=move |_| {
                store.leaving.set_value(true);
                context.flush_leave();
            }
            on:focusin=move |_| focus_in()
            on:click=move |_| focus_in()
            on:focusout=move |event: ev::FocusEvent| {
                let inside = store.viewport.get_untracked().is_some_and(|viewport| {
                    event
                        .related_target()
                        .and_then(|target| target.dyn_into::<Node>().ok())
                        .is_some_and(|target| viewport.contains(Some(&target)))
                });
                if !store.focused.get_untracked() || inside {
                    return;
                }
                store.focused.set(false);
                context.resume_if_window_focused();
            }
            on:keydown=move |event: ev::KeyboardEvent| {
                let on_viewport = store.viewport.get_untracked().is_some_and(|viewport| {
                    event
                        .target()
                        .and_then(|target| target.dyn_into::<Node>().ok())
                        .is_some_and(|target| viewport.is_same_node(Some(&target)))
                });
                if event.key() == "Tab" && event.shift_key() && on_viewport {
                    event.prevent_default();
                    context.restore_focus();
                }
            }
            on:pointerdown=move |event: ev::PointerEvent| {
                if event.pointer_type() == "touch" {
                    store.touching.set_value(true);
                }
            }
            on:pointerup=move |event: ev::PointerEvent| {
                if event.pointer_type() == "touch" {
                    store.touching.set_value(false);
                    context.flush_leave();
                }
            }
        >
            {children.map(|children| children())}
            // tab past the last toast goes back where f6 came from
            <span
                aria-hidden="true"
                tabindex="0"
                data-base-ui-focus-guard=""
                style=VISUALLY_HIDDEN
                hidden=move || empty() || store.prev_focus.with_value(Option::is_none)
                on:focus=move |_| {
                    guarding.set_value(true);
                    context.restore_focus();
                }
            ></span>
        </div>
    }
}

// a label part's id, and whether it brought its own children
type Label<E> = RwSignal<Option<(ElementId<E>, bool)>>;

#[derive(Clone, Copy)]
struct RootContext {
    toast: StoredValue<ToastObject>,
    title: Label<html::H2>,
    description: Label<html::P>,
    measure: StoredValue<Box<dyn Fn() + Send + Sync>>,
}

fn shows(own: bool, text: Option<String>) -> bool {
    own || text.is_some_and(|text| !text.is_empty())
}

fn labelled_by<E>(label: Label<E>, text: impl Fn() -> Option<String>) -> Option<String>
where
    E: html::ElementType + 'static,
    E::Output: JsCast + Clone + AsRef<HtmlElement> + 'static,
{
    label
        .get()
        .filter(|(_, own)| shows(*own, text()))
        .map(|(id, _)| id.get())
}

fn register<E>(label: Label<E>, id: ElementId<E>, own: bool)
where
    E: html::ElementType + 'static,
{
    label.set(Some((id, own)));
    on_cleanup(move || {
        if label.try_get_untracked().flatten().map(|(own, _)| own) == Some(id) {
            label.try_set(None);
        }
    });
}

#[component(element = "div", skip(role, tabindex))]
pub fn ToastRoot(
    toast: ToastObject,
    #[prop(default = vec![SwipeDirection::Down, SwipeDirection::Right])] swipe_direction: Vec<
        SwipeDirection,
    >,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<ToastContext>();
    let store = context.store();
    let id = StoredValue::new(toast.id().to_string());
    let toast = StoredValue::new(toast);
    let element = NodeRef::<html::Div>::new();
    store.roots.update_value(|roots| {
        roots.insert(id.get_value(), element);
    });
    let key = id.get_value();
    on_cleanup(move || {
        if let Some(store) = context.try_store() {
            store.roots.try_update_value(|roots| {
                roots.remove(&key);
            });
        }
    });
    let read = move |read: fn(&Stored) -> Option<String>| {
        id.with_value(|id| context.with_toast(id, read)).flatten()
    };
    let phase = Memo::new(move |_| {
        id.with_value(|id| context.with_toast(id, |toast| toast.phase))
            .flatten()
    });
    let place = Memo::new(move |_| id.with_value(|id| context.place(id)).unwrap_or_default());
    let measure = move || {
        let Some(root) = element.get_untracked() else {
            return;
        };
        let style = HtmlElement::style(&root);
        let previous = style.get_property_value("height").unwrap_or_default();
        let _ = style.set_property("height", "auto");
        let height = f64::from(root.offset_height());
        let _ = style.set_property("height", &previous);
        id.with_value(|id| context.measured(id, height));
    };
    // measured while it still has its starting style, so the browser starts the enter transition from there
    Effect::new(move |_| {
        if element.get().is_some() {
            measure();
        }
    });
    Effect::new(move |_| {
        if phase.get() == Some(Phase::Ending)
            && let Some(root) = element.get_untracked()
        {
            after_animations(root.into(), move || {
                if phase.try_get_untracked().flatten() == Some(Phase::Ending)
                    && let Some(id) = id.try_get_value()
                {
                    context.remove(&id, false);
                }
            });
        }
    });
    let swipe = Swipe::new(swipe_direction, context, id, element);
    // not delegated: a passive listener on the document couldn't stop the page scrolling under a swipe
    let touchmove = on(
        ev::Custom::<ev::TouchEvent>::new("touchmove"),
        move |event: ev::TouchEvent| {
            if swipe.directions.with_value(|all| !all.is_empty()) {
                event.prevent_default();
            }
        },
    );
    let high = move || toast.with_value(ToastObject::priority) == ToastPriority::High;
    let title = RwSignal::new(None);
    let description = RwSignal::new(None);
    let root = RootContext {
        toast,
        title,
        description,
        measure: StoredValue::new(Box::new(measure)),
    };
    view! {
        <div
            {..attrs}
            node_ref=element
            role=move || if high() { "alertdialog" } else { "dialog" }
            tabindex="0"
            aria-modal="false"
            aria-labelledby=move || labelled_by(title, move || toast.with_value(ToastObject::title))
            aria-describedby=move || labelled_by(description, move || toast.with_value(ToastObject::description))
            aria-hidden=move || (high() && !store.focused.get()).then_some("true")
            inert=move || place.get().limited
            data-expanded=move || context.expanded().then_some("")
            data-limited=move || place.get().limited.then_some("")
            data-type=move || read(|toast| toast.kind.clone())
            data-swiping=move || swipe.swiping.get().then_some("")
            data-swipe-direction=move || swipe.direction.get().map(SwipeDirection::as_str)
            data-starting-style=move || (phase.get() == Some(Phase::Starting)).then_some("")
            data-ending-style=move || (phase.get() == Some(Phase::Ending)).then_some("")
            style:--toast-swipe-movement-x=move || format!("{}px", swipe.movement().0)
            style:--toast-swipe-movement-y=move || format!("{}px", swipe.movement().1)
            style:--toast-index=move || {
                let place = place.get();
                if phase.get() == Some(Phase::Ending) { place.index } else { place.visible.unwrap_or(place.index) }
                    .to_string()
            }
            style:--toast-offset-y=move || format!("{}px", place.get().offset)
            style:--toast-height=move || {
                id.with_value(|id| context.with_toast(id, |toast| toast.height))
                    .flatten()
                    .map(|height| format!("{height}px"))
            }
            style:transition=move || swipe.swiping.get().then_some("none")
            style:transform=move || swipe.transform()
            on:pointerdown=move |event: ev::PointerEvent| swipe.down(&event)
            on:pointermove=move |event: ev::PointerEvent| swipe.moved(&event)
            on:pointerup=move |event: ev::PointerEvent| swipe.end(&event, false)
            on:pointercancel=move |event: ev::PointerEvent| swipe.end(&event, true)
            {..touchmove}
            on:keydown=move |event: ev::KeyboardEvent| {
                let inside = element.get_untracked().is_some_and(|root| {
                    document().active_element().is_some_and(|active| root.contains(Some(&active)))
                });
                if event.key() == "Escape" && inside {
                    id.with_value(|id| context.close(Some(id)));
                }
            }
        >
            <Provider value=root>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[derive(Clone, Copy)]
struct Swipe {
    directions: StoredValue<Vec<SwipeDirection>>,
    context: ToastContext,
    id: StoredValue<String>,
    element: NodeRef<html::Div>,
    swiping: RwSignal<bool>,
    direction: RwSignal<Option<SwipeDirection>>,
    drag: RwSignal<(f64, f64)>,
    initial: RwSignal<(f64, f64, f64)>,
    pointer: StoredValue<Option<i32>>,
    start: StoredValue<(f64, f64)>,
    baseline: StoredValue<(f64, f64)>,
    intended: StoredValue<Option<SwipeDirection>>,
    furthest: StoredValue<f64>,
    cancelled: StoredValue<bool>,
    real: StoredValue<bool>,
    locked: StoredValue<Option<Orientation>>,
    first_move: StoredValue<bool>,
    release: StoredValue<Option<(WindowListenerHandle, WindowListenerHandle)>>,
}

impl Swipe {
    fn new(
        directions: Vec<SwipeDirection>,
        context: ToastContext,
        id: StoredValue<String>,
        element: NodeRef<html::Div>,
    ) -> Self {
        let this = Self {
            directions: StoredValue::new(directions),
            context,
            id,
            element,
            swiping: RwSignal::new(false),
            direction: RwSignal::new(None),
            drag: RwSignal::new((0.0, 0.0)),
            initial: RwSignal::new((0.0, 0.0, 1.0)),
            pointer: StoredValue::new(None),
            start: StoredValue::new((0.0, 0.0)),
            baseline: StoredValue::new((0.0, 0.0)),
            intended: StoredValue::new(None),
            furthest: StoredValue::new(0.0),
            cancelled: StoredValue::new(false),
            real: StoredValue::new(false),
            locked: StoredValue::new(None),
            first_move: StoredValue::new(false),
            release: StoredValue::new(None),
        };
        on_cleanup(move || this.stop_listening());
        this
    }

    fn allows(self, direction: SwipeDirection) -> bool {
        self.directions.with_value(|all| all.contains(&direction))
    }

    fn both_ways(self) -> bool {
        (self.allows(SwipeDirection::Left) && self.allows(SwipeDirection::Right))
            || (self.allows(SwipeDirection::Up) && self.allows(SwipeDirection::Down))
    }

    fn movement(self) -> (f64, f64) {
        let (x, y) = self.drag.get();
        let (from_x, from_y, _) = self.initial.get();
        (x - from_x, y - from_y)
    }

    fn transform(self) -> Option<String> {
        self.swiping.get().then(|| {
            let (x, y) = self.drag.get();
            let (_, _, scale) = self.initial.get();
            format!("translateX({x}px) translateY({y}px) scale({scale})")
        })
    }

    fn stop_listening(self) {
        if let Some(Some((up, cancel))) = self.release.try_update_value(Option::take) {
            up.remove();
            cancel.remove();
        }
    }

    fn down(self, event: &ev::PointerEvent) {
        if event.button() != 0 || self.directions.with_value(Vec::is_empty) {
            return;
        }
        if event.pointer_type() == "touch" {
            self.context.pause();
        }
        let ignored = event
            .target()
            .and_then(|target| target.dyn_into::<Element>().ok())
            .and_then(|target| target.closest(SWIPE_IGNORE).ok().flatten())
            .is_some();
        if ignored {
            return;
        }
        let Some(root) = self.element.get_untracked() else {
            return;
        };
        let at = (f64::from(event.client_x()), f64::from(event.client_y()));
        self.cancelled.set_value(false);
        self.intended.set_value(None);
        self.furthest.set_value(0.0);
        self.pointer.set_value(Some(event.pointer_id()));
        self.start.set_value(at);
        self.baseline.set_value(at);
        let transform = transform_of(&root);
        self.initial.set(transform);
        self.drag.set((transform.0, transform.1));
        if let Some(store) = self.context.try_store() {
            store.hovering.set(true);
        }
        self.swiping.set(true);
        self.real.set_value(false);
        self.locked.set_value(None);
        self.first_move.set_value(true);
        self.stop_listening();
        let up = window_event_listener(ev::pointerup, move |event| self.end(&event, false));
        let cancel = window_event_listener(ev::pointercancel, move |event| self.end(&event, true));
        self.release.set_value(Some((up, cancel)));
        let _ = root.set_pointer_capture(event.pointer_id());
    }

    fn moved(self, event: &ev::PointerEvent) {
        if self.pointer.get_value() != Some(event.pointer_id()) {
            return;
        }
        event.prevent_default();
        let (x, y) = (f64::from(event.client_x()), f64::from(event.client_y()));
        if self.first_move.get_value() {
            self.start.set_value((x, y));
            self.first_move.set_value(false);
        }
        let (movement_x, movement_y) =
            (f64::from(event.movement_x()), f64::from(event.movement_y()));
        self.baseline.update_value(|(base_x, base_y)| {
            if (movement_y < 0.0 && y > *base_y) || (movement_y > 0.0 && y < *base_y) {
                *base_y = y;
            }
            if (movement_x < 0.0 && x > *base_x) || (movement_x > 0.0 && x < *base_x) {
                *base_x = x;
            }
        });
        let (start_x, start_y) = self.start.get_value();
        let (base_x, base_y) = self.baseline.get_value();
        let (delta_x, delta_y) = (x - start_x, y - start_y);
        let horizontal = self.allows(SwipeDirection::Left) || self.allows(SwipeDirection::Right);
        let vertical = self.allows(SwipeDirection::Up) || self.allows(SwipeDirection::Down);
        let mut locked = self.locked.get_value();
        if !self.real.get_value() && delta_x.hypot(delta_y) >= 1.0 {
            self.real.set_value(true);
            if horizontal && vertical {
                locked = Some(if delta_x.abs() > delta_y.abs() {
                    Orientation::Horizontal
                } else {
                    Orientation::Vertical
                });
                self.locked.set_value(locked);
            }
        }
        if let Some(direction) = self.intended.get_value() {
            let current = displacement(direction, x - base_x, y - base_y);
            if current > SWIPE_THRESHOLD {
                self.cancelled.set_value(false);
                self.direction.set(Some(direction));
            } else if !self.both_ways()
                && self.furthest.get_value() - current >= REVERSE_CANCEL_THRESHOLD
            {
                self.cancelled.set_value(true);
            }
        } else {
            let candidate = match locked {
                Some(Orientation::Vertical) if delta_y > 0.0 => Some(SwipeDirection::Down),
                Some(Orientation::Vertical) if delta_y < 0.0 => Some(SwipeDirection::Up),
                Some(Orientation::Horizontal) if delta_x > 0.0 => Some(SwipeDirection::Right),
                Some(Orientation::Horizontal) if delta_x < 0.0 => Some(SwipeDirection::Left),
                Some(_) => None,
                None if delta_x.abs() >= delta_y.abs() => Some(if delta_x > 0.0 {
                    SwipeDirection::Right
                } else {
                    SwipeDirection::Left
                }),
                None => Some(if delta_y > 0.0 {
                    SwipeDirection::Down
                } else {
                    SwipeDirection::Up
                }),
            };
            if let Some(candidate) = candidate
                && self.allows(candidate)
            {
                self.intended.set_value(Some(candidate));
                self.furthest
                    .set_value(displacement(candidate, delta_x, delta_y));
                self.direction.set(Some(candidate));
            }
        }
        let damp = |delta: f64, allowed: bool| {
            if allowed {
                delta
            } else {
                delta.signum() * delta.abs().powf(OPPOSITE_DIRECTION_DAMPING)
            }
        };
        let damped_x = damp(
            delta_x,
            (delta_x > 0.0 && self.allows(SwipeDirection::Right))
                || (delta_x < 0.0 && self.allows(SwipeDirection::Left))
                || delta_x == 0.0,
        );
        let damped_y = damp(
            delta_y,
            (delta_y > 0.0 && self.allows(SwipeDirection::Down))
                || (delta_y < 0.0 && self.allows(SwipeDirection::Up))
                || delta_y == 0.0,
        );
        let (from_x, from_y, _) = self.initial.get_untracked();
        let x = if locked != Some(Orientation::Vertical) && horizontal {
            from_x + damped_x
        } else {
            from_x
        };
        let y = if locked != Some(Orientation::Horizontal) && vertical {
            from_y + damped_y
        } else {
            from_y
        };
        self.drag.set((x, y));
    }

    fn end(self, event: &ev::PointerEvent, cancel: bool) {
        if self.pointer.get_value() != Some(event.pointer_id()) {
            return;
        }
        self.pointer.set_value(None);
        self.stop_listening();
        self.swiping.set(false);
        self.real.set_value(false);
        self.locked.set_value(None);
        let (from_x, from_y, _) = self.initial.get_untracked();
        if cancel || self.cancelled.get_value() {
            self.drag.set((from_x, from_y));
            self.direction.set(None);
            return;
        }
        let (x, y) = self.drag.get_untracked();
        let (delta_x, delta_y) = (x - from_x, y - from_y);
        let dismiss = self.directions.with_value(|all| {
            all.iter()
                .copied()
                .find(|direction| displacement(*direction, delta_x, delta_y) > SWIPE_THRESHOLD)
        });
        if let Some(direction) = dismiss {
            self.direction.set(Some(direction));
            self.id.with_value(|id| self.context.close(Some(id)));
        } else {
            self.drag.set((from_x, from_y));
            self.direction.set(None);
        }
    }
}

#[component(element = "div")]
pub fn ToastContent(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<ToastContext>();
    let root = expect_context::<RootContext>();
    let element = NodeRef::<html::Div>::new();
    let behind = move || {
        root.toast
            .with_value(|toast| context.place(toast.id()))
            .and_then(|place| place.visible)
            .is_some_and(|visible| visible > 0)
    };
    Effect::new(move |_| {
        if let Some(content) = element.get() {
            // a frame later: the height it sets resizes the content again, which webkit reports as a loop
            observe_resize(&content, move || {
                request_animation_frame(move || {
                    root.measure.try_with_value(|measure| measure());
                });
            });
        }
    });
    view! {
        <div
            {..attrs}
            node_ref=element
            data-behind=move || behind().then_some("")
            data-expanded=move || context.expanded().then_some("")
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "h2", skip(id))]
pub fn ToastTitle(
    #[prop(optional_no_strip, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let root = expect_context::<RootContext>();
    let element = NodeRef::new();
    let id = ElementId::new(element, id.unwrap_or_else(use_id));
    let own = children.is_some();
    register(root.title, id, own);
    let content = match children {
        Some(children) => Either::Left(children()),
        None => Either::Right(move || root.toast.with_value(ToastObject::title)),
    };
    view! {
        <h2
            {..attrs}
            node_ref=element
            id=id.get()
            hidden=move || !shows(own, root.toast.with_value(ToastObject::title))
            data-type=move || root.toast.with_value(ToastObject::r#type)
        >
            {content}
        </h2>
    }
}

#[component(element = "p", skip(id))]
pub fn ToastDescription(
    #[prop(optional_no_strip, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let root = expect_context::<RootContext>();
    let element = NodeRef::new();
    let id = ElementId::new(element, id.unwrap_or_else(use_id));
    let own = children.is_some();
    register(root.description, id, own);
    let content = match children {
        Some(children) => Either::Left(children()),
        None => Either::Right(move || root.toast.with_value(ToastObject::description)),
    };
    view! {
        <p
            {..attrs}
            node_ref=element
            id=id.get()
            hidden=move || !shows(own, root.toast.with_value(ToastObject::description))
            data-type=move || root.toast.with_value(ToastObject::r#type)
        >
            {content}
        </p>
    }
}

#[component(element = "button", skip(r#type))]
pub fn ToastAction(
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional_no_strip, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let root = expect_context::<RootContext>();
    let action = move || {
        root.toast
            .with_value(|toast| toast.read(|toast| toast.action.clone()))
            .flatten()
    };
    let own = children.is_some();
    let label = match children {
        Some(children) => Either::Left(children()),
        None => Either::Right(move || action().map(|action| action.label)),
    };
    view! {
        <button
            {..attrs}
            {..button_attributes(move || disabled.get(), || false, tabindex)}
            type="button"
            hidden=move || !own && action().is_none()
            data-type=move || root.toast.with_value(ToastObject::r#type)
            on:click=move |_| {
                if let Some(on_click) = untrack(action).and_then(|action| action.on_click) {
                    on_click.run(root.toast.with_value(|toast| toast.id().to_string()));
                }
            }
        >
            {label}
        </button>
    }
}

#[component(element = "button", skip(r#type))]
pub fn ToastClose(
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional_no_strip, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let root = expect_context::<RootContext>();
    let context = expect_context::<ToastContext>();
    let focused = RwSignal::new(false);
    view! {
        <button
            {..attrs}
            {..button_attributes(move || disabled.get(), || false, tabindex)}
            type="button"
            aria-hidden=move || (!context.expanded() && !focused.get()).to_string()
            data-type=move || root.toast.with_value(ToastObject::r#type)
            on:click=move |_| root.toast.with_value(|toast| context.close(Some(toast.id())))
            on:focus=move |_| focused.set(true)
            on:blur=move |_| focused.set(false)
        >
            {children.map(|children| children())}
        </button>
    }
}
