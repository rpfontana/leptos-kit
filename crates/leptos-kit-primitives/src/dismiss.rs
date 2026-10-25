use crate::ChangeReason;
use leptos::ev;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use web_sys::Node;

// base ui's dismiss for a popup the browser doesn't light-dismiss: esc, or a press outside its parts
pub(crate) fn dismiss(
    open: impl Fn() -> bool + 'static,
    inside: impl Fn(&Node) -> bool + Copy + 'static,
    close: impl Fn(ChangeReason) + Copy + 'static,
) {
    let listeners = StoredValue::new(Vec::<WindowListenerHandle>::new());
    let stop = move || {
        if let Some(listeners) = listeners.try_update_value(std::mem::take) {
            listeners.into_iter().for_each(WindowListenerHandle::remove);
        }
    };
    Effect::new(move || {
        stop();
        if !open() {
            return;
        }
        let escape = window_event_listener(ev::keydown, move |event| {
            if event.key() == "Escape" {
                close(ChangeReason::EscapeKey);
            }
        });
        let press = window_event_listener(ev::pointerdown, move |event| {
            let target = event
                .target()
                .and_then(|target| target.dyn_into::<Node>().ok());
            if target.is_some_and(|target| !inside(&target)) {
                close(ChangeReason::OutsidePress);
            }
        });
        listeners.set_value(vec![escape, press]);
    });
    on_cleanup(stop);
}
