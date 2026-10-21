use crate::button::button_attributes;
use crate::id::ElementId;
use crate::{Controlled, Native, NativeOpen, reason, use_id};
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_kit_macros::component;

#[derive(Clone, Copy)]
struct DialogContext {
    id: ElementId<html::Dialog>,
    native: NativeOpen<html::Dialog>,
    dialog: NodeRef<html::Dialog>,
    popup: NodeRef<html::Div>,
    trigger: NodeRef<html::Button>,
    modal: Signal<bool>,
    disable_pointer_dismissal: Signal<bool>,
    role: &'static str,
    title: RwSignal<Option<String>>,
    description: RwSignal<Option<String>>,
}

impl DialogContext {
    fn open(&self) -> bool {
        self.native.state().get()
    }
}

#[leptos::component]
pub fn DialogRoot(
    #[prop(optional, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, &'static str)>>,
    #[prop(into, default = true.into())] modal: Signal<bool>,
    #[prop(optional, into)] disable_pointer_dismissal: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let state = Controlled::new(open, default_open, on_open_change);
    root(state, modal, disable_pointer_dismissal, "dialog", children)
}

// base ui's alert dialog: always modal, and a click outside doesn't close it
#[leptos::component]
pub fn AlertDialogRoot(
    #[prop(optional, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, &'static str)>>,
    children: Children,
) -> impl IntoView {
    let state = Controlled::new(open, default_open, on_open_change);
    root(state, true.into(), true.into(), "alertdialog", children)
}

fn root(
    state: Controlled<bool>,
    modal: Signal<bool>,
    disable_pointer_dismissal: Signal<bool>,
    role: &'static str,
    children: Children,
) -> impl IntoView {
    let dialog = NodeRef::new();
    let popup = NodeRef::new();
    let kind = Signal::derive(move || {
        if modal.get() {
            Native::Modal
        } else {
            Native::Dialog
        }
    });
    let context = DialogContext {
        id: ElementId::new(dialog, use_id()),
        native: NativeOpen::new(kind, dialog, state, Some(popup)),
        dialog,
        popup,
        trigger: NodeRef::new(),
        modal,
        disable_pointer_dismissal,
        role,
        title: RwSignal::new(None),
        description: RwSignal::new(None),
    };
    // components share their parent's owner, a sibling root would overwrite the context
    view! { <Provider value=context>{children()}</Provider> }
}

#[component(element = "button", skip(r#type))]
pub fn DialogTrigger(
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<DialogContext>();
    let disabled = move || disabled.is_some_and(|disabled| disabled.get());
    let open = move || context.open();

    view! {
        <button
            {..attrs}
            {..button_attributes(disabled, || false, tabindex)}
            node_ref=context.trigger
            type="button"
            // no invoker command for show()
            commandfor=move || context.modal.get().then(|| context.id.get())
            command=move || context.modal.get().then_some("show-modal")
            aria-haspopup="dialog"
            aria-expanded=move || open().to_string()
            aria-controls=move || open().then(|| context.id.get())
            data-popup-open=move || open().then_some("")
            on:click=move |_| {
                if context.modal.get_untracked() {
                    context.native.expect(reason::TRIGGER_PRESS);
                } else {
                    context.native.state().set(true, reason::TRIGGER_PRESS);
                }
            }
        >
            {children.map(|children| children())}
        </button>
    }
}

#[component(element = "dialog", skip(id, open, closedby, style, role))]
pub fn DialogPortal(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<DialogContext>();
    let open = move || context.open();
    // a dialog removed while open never fires close
    on_cleanup(move || {
        #[cfg(not(feature = "ssr"))]
        if context.modal.try_get_untracked().unwrap_or(false) {
            release_gutter();
        }
    });
    // cancel fires for esc and close watchers alike
    let escape = StoredValue::new(false);

    // a native close would skip the exit animation
    let request_close = move |event: &web_sys::Event, reason: &'static str| {
        if event.cancelable() {
            event.prevent_default();
            context.native.state().set(false, reason);
        } else {
            context.native.expect(reason);
        }
    };
    let measure = leptos::tachys::html::event::on(
        ev::Custom::<web_sys::ToggleEvent>::new("beforetoggle"),
        move |event: web_sys::ToggleEvent| {
            if event.new_state() == "open" && context.modal.try_get_untracked() == Some(true) {
                keep_gutter();
            }
        },
    );

    view! {
        <dialog
            {..attrs}
            {..measure}
            node_ref=context.dialog
            id=context.id.get()
            role=(context.role != "dialog").then_some(context.role)
            style="position:fixed;inset:0;width:auto;height:auto;max-width:none;max-height:none;margin:0;padding:0;border:0;background:transparent;overflow:visible"
            style:pointer-events=move || (!context.modal.get()).then_some("none")
            aria-labelledby=move || context.title.get()
            aria-describedby=move || context.description.get()
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            on:keydown=move |event: ev::KeyboardEvent| {
                if event.key() == "Escape" {
                    escape.set_value(true);
                }
            }
            on:cancel=move |event| {
                let key = escape.try_update_value(std::mem::take).unwrap_or_default();
                request_close(&event, if key { reason::ESCAPE_KEY } else { reason::CLOSE_WATCHER });
            }
            on:command=move |event: web_sys::Event| {
                let command = js_string(&event, "command");
                if command.as_deref() == Some("close") || command.as_deref() == Some("request-close") {
                    request_close(&event, reason::CLOSE_PRESS);
                }
            }
            on:toggle=move |_| sync(context)
            on:close=move |_| sync(context)
        >
            {children.map(|children| children())}
        </dialog>
    }
}

#[component(element = "div", skip(role))]
pub fn DialogBackdrop() -> impl IntoView {
    let context = expect_context::<DialogContext>();
    let open = move || context.open();
    view! {
        <div
            {..attrs}
            role="presentation"
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            on:click=move |_| {
                if !context.disable_pointer_dismissal.get_untracked() {
                    context.native.state().set(false, reason::OUTSIDE_PRESS);
                }
            }
        ></div>
    }
}

#[component(element = "div")]
pub fn DialogPopup(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<DialogContext>();
    let open = move || context.open();
    view! {
        <div
            {..attrs}
            node_ref=context.popup
            style:pointer-events="auto"
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "button", skip(r#type))]
pub fn DialogClose(
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<DialogContext>();
    let disabled = move || disabled.is_some_and(|disabled| disabled.get());
    view! {
        <button
            {..attrs}
            {..button_attributes(disabled, || false, tabindex)}
            type="button"
            commandfor=context.id.get()
            command="close"
        >
            {children.map(|children| children())}
        </button>
    }
}

#[component(element = "h2")]
pub fn DialogTitle(
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<DialogContext>();
    let id = id.unwrap_or_else(|| format!("{}-title", context.id.get()));
    context.title.set(Some(id.clone()));
    on_cleanup(move || context.title.set(None));
    view! {
        <h2 {..attrs} id=id>
            {children.map(|children| children())}
        </h2>
    }
}

#[component(element = "p")]
pub fn DialogDescription(
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<DialogContext>();
    let id = id.unwrap_or_else(|| format!("{}-description", context.id.get()));
    context.description.set(Some(id.clone()));
    on_cleanup(move || context.description.set(None));
    view! {
        <p {..attrs} id=id>
            {children.map(|children| children())}
        </p>
    }
}

fn sync(context: DialogContext) {
    context.native.sync(reason::NONE);
    if context.native.state().try_get_untracked() == Some(false) {
        release_gutter();
        return_focus(context);
    }
}

// webkit doesn't focus a clicked button, so focus return has nowhere to go
fn return_focus(context: DialogContext) {
    let (Some(popup), Some(trigger)) = (
        context.dialog.get_untracked(),
        context.trigger.get_untracked(),
    ) else {
        return;
    };
    let stranded = match document().active_element() {
        Some(active) => {
            popup.contains(Some(&active)) || active.tag_name().eq_ignore_ascii_case("body")
        }
        None => true,
    };
    if stranded {
        let _ = trigger.focus();
    }
}

// CommandEvent isn't in web-sys yet
fn js_string(event: &web_sys::Event, key: &str) -> Option<String> {
    web_sys::js_sys::Reflect::get(event, &key.into())
        .ok()
        .and_then(|value| value.as_string())
}

// measured before overflow: hidden takes the scrollbar away
fn keep_gutter() {
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

fn release_gutter() {
    if let Some(html) = document().document_element() {
        let html: &web_sys::HtmlElement = html.unchecked_ref();
        let _ = html.style().remove_property("scrollbar-gutter");
    }
}
