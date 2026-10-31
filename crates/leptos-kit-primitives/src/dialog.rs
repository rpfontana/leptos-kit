use crate::button::button_attributes;
use crate::id::ElementId;
use crate::native::{after_paint, keep_gutter, release_gutter};
use crate::transition::{TransitionStatus, transition_attributes};
use crate::{ChangeReason, Controlled, Native, NativeOpen, use_id};
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_kit_macros::component;
use web_sys::HtmlElement;

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
    phase: RwSignal<TransitionStatus>,
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
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, ChangeReason)>>,
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
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, ChangeReason)>>,
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
        phase: RwSignal::new(TransitionStatus::Idle),
        title: RwSignal::new(None),
        description: RwSignal::new(None),
    };
    // the native close waits for what the ending style starts
    Effect::new(move |was_open: Option<bool>| {
        let open = state.get();
        if was_open == Some(true) && !open {
            context.phase.set(TransitionStatus::Ending);
        }
        open
    });
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
                    context.native.expect(ChangeReason::TriggerPress);
                } else {
                    context.native.state().set(true, ChangeReason::TriggerPress);
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
            let dialog = context.dialog.try_get_untracked().flatten();
            release_gutter(dialog.as_ref().map(AsRef::as_ref));
        }
    });
    // cancel fires for esc and close watchers alike
    let escape = StoredValue::new(false);

    // a native close would skip the exit animation
    let request_close = move |event: &web_sys::Event, reason: ChangeReason| {
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
            if event.new_state() != "open" {
                return;
            }
            if context.modal.try_get_untracked() == Some(true) {
                keep_gutter();
            }
            // set before the dialog shows, so its first frame already has the starting style
            context.phase.set(TransitionStatus::Starting);
            after_paint(move || {
                if context.phase.try_get_untracked() == Some(TransitionStatus::Starting) {
                    context.phase.set(TransitionStatus::Idle);
                }
            });
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
                request_close(&event, if key { ChangeReason::EscapeKey } else { ChangeReason::CloseWatcher });
            }
            on:command=move |event: web_sys::Event| {
                let command = js_string(&event, "command");
                if command.as_deref() == Some("close") || command.as_deref() == Some("request-close") {
                    request_close(&event, ChangeReason::ClosePress);
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
            {..transition_attributes(context.phase)}
            role="presentation"
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            on:click=move |_| {
                if !context.disable_pointer_dismissal.get_untracked() {
                    context.native.state().set(false, ChangeReason::OutsidePress);
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
            {..transition_attributes(context.phase)}
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
    context.native.sync(ChangeReason::None);
    match context.native.state().try_get_untracked() {
        Some(false) => {
            context.phase.try_set(TransitionStatus::Idle);
            let dialog = context.dialog.try_get_untracked().flatten();
            release_gutter(dialog.as_ref().map(AsRef::as_ref));
            return_focus(context);
        }
        Some(true) => focus_tabbable(context),
        None => {}
    }
}

// the native dialog focuses its first focusable element even at tabindex -1, base ui its first tabbable one
fn focus_tabbable(context: DialogContext) {
    let Some(dialog) = context.dialog.try_get_untracked().flatten() else {
        return;
    };
    let Some(active) = document()
        .active_element()
        .and_then(|active| active.dyn_into::<HtmlElement>().ok())
    else {
        return;
    };
    if active.tab_index() >= 0 || !dialog.contains(Some(&active)) {
        return;
    }
    let Ok(candidates) = dialog.query_selector_all(
        "a[href], button, input, select, textarea, [tabindex], [contenteditable]",
    ) else {
        return;
    };
    let tabbable = (0..candidates.length())
        .filter_map(|at| candidates.item(at)?.dyn_into::<HtmlElement>().ok())
        .find(|element| {
            element.tab_index() >= 0 && !element.matches(":disabled").unwrap_or(false) && {
                let rect = element.get_bounding_client_rect();
                rect.width() > 0.0 || rect.height() > 0.0
            }
        });
    if let Some(element) = tabbable {
        let _ = element.focus();
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
