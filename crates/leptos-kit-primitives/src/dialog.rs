use crate::button::button_attributes;
use crate::{Controlled, Native, NativeOpen, reason, use_id};
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_kit_macros::component;

#[derive(Clone, Copy)]
struct DialogContext {
    id: StoredValue<String>,
    native: NativeOpen<html::Dialog>,
    dialog: NodeRef<html::Dialog>,
    popup: NodeRef<html::Div>,
    trigger: NodeRef<html::Button>,
    modal: bool,
    pointer_dismissal: bool,
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
    #[prop(default = true)] modal: bool,
    #[prop(optional)] disable_pointer_dismissal: bool,
    children: Children,
) -> impl IntoView {
    let dialog = NodeRef::new();
    let popup = NodeRef::new();
    let state = Controlled::new(open, default_open, on_open_change);
    let kind = if modal { Native::Modal } else { Native::Dialog };
    provide_context(DialogContext {
        id: StoredValue::new(use_id()),
        native: NativeOpen::new(kind, dialog, state, Some(popup)),
        dialog,
        popup,
        trigger: NodeRef::new(),
        modal,
        pointer_dismissal: !disable_pointer_dismissal,
        title: RwSignal::new(None),
        description: RwSignal::new(None),
    });
    children()
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
            {..button_attributes(disabled, false, tabindex)}
            node_ref=context.trigger
            type="button"
            // no invoker command for show()
            commandfor=context.modal.then(|| context.id.get_value())
            command=context.modal.then_some("show-modal")
            aria-haspopup="dialog"
            aria-expanded=move || open().to_string()
            aria-controls=move || open().then(|| context.id.get_value())
            data-popup-open=move || open().then_some("")
            on:click=move |_| {
                if context.modal {
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

#[component(element = "dialog", skip(id, open, closedby, style))]
pub fn DialogPortal(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<DialogContext>();
    let open = move || context.open();
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
            if context.modal && event.new_state() == "open" {
                keep_gutter();
            }
        },
    );

    view! {
        <dialog
            {..attrs}
            {..measure}
            node_ref=context.dialog
            id=context.id.get_value()
            style="position:fixed;inset:0;width:auto;height:auto;max-width:none;max-height:none;margin:0;padding:0;border:0;background:transparent;overflow:visible"
            style:pointer-events=(!context.modal).then_some("none")
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
                if context.pointer_dismissal {
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
            {..button_attributes(disabled, false, tabindex)}
            type="button"
            commandfor=context.id.get_value()
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
    let id = id.unwrap_or_else(|| format!("{}-title", context.id.get_value()));
    context.title.set(Some(id.clone()));
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
    let id = id.unwrap_or_else(|| format!("{}-description", context.id.get_value()));
    context.description.set(Some(id.clone()));
    view! {
        <p {..attrs} id=id>
            {children.map(|children| children())}
        </p>
    }
}

fn sync(context: DialogContext) {
    context.native.sync(reason::NONE);
    if !context.native.state().get_untracked() {
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
