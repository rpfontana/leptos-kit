use crate::native::observe_resize;
use crate::{ChangeReason, Controlled, Keyword, keyword};
use leptos::attr::Attribute;
use leptos::context::Provider;
use leptos::html;
use leptos::prelude::*;
use leptos::tachys::html::node_ref::node_ref;
use leptos::tachys::html::property::prop;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;
use leptos_kit_macros::component;
use std::time::Duration;
use web_sys::HtmlInputElement;

pub const REGEXP_ONLY_DIGITS: &str = "^\\d+$";
pub const REGEXP_ONLY_CHARS: &str = "^[a-zA-Z]+$";
pub const REGEXP_ONLY_DIGITS_AND_CHARS: &str = "^[a-zA-Z0-9]+$";

// the box over the slots that holds the input, where a password manager's badge lands without moving them
pub const OTP_INPUT_WRAPPER_STYLE: &str = "position: absolute; inset: 0; pointer-events: none;";

const INPUT_STYLE: &str = "position: absolute; inset: 0; width: 100%; height: 100%; display: flex; \
    text-align: left; opacity: 1; color: transparent; pointer-events: all; background: transparent; \
    caret-color: transparent; border: 0 solid transparent; outline: 0 solid transparent; box-shadow: none; \
    line-height: 1; letter-spacing: -.5em; font-size: var(--root-height); font-family: monospace; \
    font-variant-numeric: tabular-nums";

const NOSCRIPT_CSS_FALLBACK: &str = "<style>
[data-input-otp] {
  --nojs-bg: white !important;
  --nojs-fg: black !important;

  background-color: var(--nojs-bg) !important;
  color: var(--nojs-fg) !important;
  caret-color: var(--nojs-fg) !important;
  letter-spacing: .25em !important;
  text-align: center !important;
  border: 1px solid var(--nojs-fg) !important;
  border-radius: 4px !important;
  width: 100% !important;
}
@media (prefers-color-scheme: dark) {
  [data-input-otp] {
    --nojs-bg: black !important;
    --nojs-fg: white !important;
  }
}</style>";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Keyword)]
pub enum InputMode {
    None,
    Text,
    Decimal,
    #[default]
    Numeric,
    Tel,
    Search,
    Email,
    Url,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct OtpSlot {
    pub char: Option<char>,
    pub placeholder_char: Option<char>,
    pub is_active: bool,
    pub has_fake_caret: bool,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Selection {
    start: Option<u32>,
    end: Option<u32>,
}

#[derive(Clone, Copy)]
struct Otp {
    value: Controlled<String>,
    max_length: usize,
    pattern: Option<Signal<String>>,
    input_mode: Signal<InputMode>,
    placeholder: Option<Signal<String>>,
    autocomplete: Option<Signal<String>>,
    disabled: Signal<bool>,
    on_complete: Option<Callback<String>>,
    paste_transformer: Option<Callback<String, String>>,
    input: NodeRef<html::Input>,
    focused: RwSignal<bool>,
    hovering: RwSignal<bool>,
    // the slots the input's selection covers, as the page last saw it
    mirror: RwSignal<Option<(u32, u32)>>,
    previous: StoredValue<Selection>,
}

#[derive(Clone, Copy)]
pub struct OtpContext(StoredValue<Otp>);

impl std::fmt::Debug for OtpContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OtpContext").finish_non_exhaustive()
    }
}

pub fn use_otp() -> OtpContext {
    expect_context::<OtpContext>()
}

impl OtpContext {
    fn try_otp(self) -> Option<Otp> {
        self.0.try_get_value()
    }

    pub fn slot(self, index: usize) -> OtpSlot {
        let Some(otp) = self.try_otp() else {
            return OtpSlot::default();
        };
        let (char, empty) = otp
            .value
            .with(|value| (value.chars().nth(index), value.is_empty()));
        let placeholder_char = otp
            .placeholder
            .filter(|_| empty)
            .and_then(|placeholder| placeholder.with(|text| text.chars().nth(index)));
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        let is_active = otp.focused.get()
            && otp.mirror.get().is_some_and(|(start, end)| {
                (start == end && index == start) || (index >= start && index < end)
            });
        OtpSlot {
            char,
            placeholder_char,
            is_active,
            has_fake_caret: is_active && char.is_none(),
        }
    }

    pub fn is_focused(self) -> bool {
        self.try_otp().is_some_and(|otp| otp.focused.get())
    }

    pub fn is_hovering(self) -> bool {
        self.try_otp()
            .is_some_and(|otp| !otp.disabled.get() && otp.hovering.get())
    }

    fn accepts(otp: &Otp, value: &str) -> bool {
        value.is_empty()
            || otp.pattern.is_none_or(|pattern| {
                web_sys::js_sys::RegExp::new(&pattern.get_untracked(), "").test(value)
            })
    }

    fn commit(self, next: String, reason: ChangeReason, input: &HtmlInputElement) {
        let Some(otp) = self.try_otp() else {
            return;
        };
        let was = otp.value.with_untracked(|value| value.chars().count());
        let length = next.chars().count();
        otp.value.set(next.clone(), reason);
        if was < otp.max_length
            && length == otp.max_length
            && let Some(on_complete) = otp.on_complete
        {
            on_complete.run(next);
        }
        // like react, a controlled input goes back to its value unless the caller took the new one
        let value = otp.value.get_untracked();
        if input.value() != value {
            input.set_value(&value);
        }
    }

    fn changed(self, input: &HtmlInputElement) {
        let Some(otp) = self.try_otp() else {
            return;
        };
        let next: String = input.value().chars().take(otp.max_length).collect();
        if !Self::accepts(&otp, &next) {
            input.set_value(&otp.value.get_untracked());
            return;
        }
        // a deletion fires no selectionchange of its own
        if next.encode_utf16().count()
            < otp
                .value
                .with_untracked(|value| value.encode_utf16().count())
            && let Ok(event) = web_sys::Event::new("selectionchange")
        {
            let _ = document().dispatch_event(&event);
        }
        self.commit(next, ChangeReason::InputChange, input);
    }

    fn pasted(self, event: &web_sys::ClipboardEvent) {
        let Some(otp) = self.try_otp() else {
            return;
        };
        let Some(input) = otp.input.get_untracked() else {
            return;
        };
        // otherwise the browser pastes into the input, as it does for typing
        if otp.paste_transformer.is_none() && !is_ios() {
            return;
        }
        let Some(data) = event.clipboard_data() else {
            return;
        };
        let content = data.get_data("text/plain").unwrap_or_default();
        let content = match otp.paste_transformer {
            Some(transformer) => transformer.run(content),
            None => content,
        };
        event.prevent_default();

        let value: Vec<u16> = otp
            .value
            .with_untracked(|value| value.encode_utf16().collect());
        let at = |offset: Option<u32>| {
            offset.map_or(value.len(), |offset| {
                usize::try_from(offset).map_or(value.len(), |offset| offset.min(value.len()))
            })
        };
        let start = at(input.selection_start().ok().flatten());
        let end = at(input.selection_end().ok().flatten()).max(start);
        let mut next: Vec<u16> = value[..start].to_vec();
        next.extend(content.encode_utf16());
        next.extend_from_slice(&value[end..]);
        next.truncate(otp.max_length);
        let next = String::from_utf16_lossy(&next);
        if !Self::accepts(&otp, &next) {
            return;
        }

        input.set_value(&next);
        let length = u32::try_from(next.encode_utf16().count()).unwrap_or(u32::MAX);
        self.commit(next, ChangeReason::InputPaste, &input);
        let start = length.min(last(otp.max_length));
        let _ = input.set_selection_range(start, length);
        otp.mirror.set(Some((start, length)));
    }

    fn focused(self) {
        let Some(otp) = self.try_otp() else {
            return;
        };
        if let Some(input) = otp.input.get_untracked() {
            let length = u32::try_from(input.value().encode_utf16().count()).unwrap_or(u32::MAX);
            let start = length.min(last(otp.max_length));
            let _ = input.set_selection_range(start, length);
            otp.mirror.set(Some((start, length)));
        }
        otp.focused.set(true);
    }

    // a caret on a filled slot selects that slot's character, so typing replaces it
    fn selection_changed(self) {
        let Some(otp) = self.try_otp() else {
            return;
        };
        let Some(input) = otp.input.get_untracked() else {
            return;
        };
        let focused = document()
            .active_element()
            .is_some_and(|active| active == **input);
        if !focused {
            otp.mirror.set(None);
            return;
        }

        let start = input.selection_start().ok().flatten();
        let end = input.selection_end().ok().flatten();
        let length = u32::try_from(input.value().encode_utf16().count()).unwrap_or(u32::MAX);
        let max = u32::try_from(otp.max_length).unwrap_or(u32::MAX);
        let previous = otp.previous.get_value();

        let mut picked: Option<(u32, u32)> = None;
        let mut picked_direction: Option<&'static str> = None;
        if let (Some(start), Some(end)) = (start, end)
            && length != 0
            && start == end
            && !(start == length && length < max)
        {
            let caret = start;
            if caret == 0 {
                picked = Some((0, 1));
                picked_direction = Some("forward");
            } else if caret == max {
                picked = Some((caret - 1, caret));
                picked_direction = Some("backward");
            } else if max > 1 && length > 1 {
                let mut offset = 0;
                if let (Some(was_start), Some(was_end)) = (previous.start, previous.end) {
                    let backward = caret < was_end;
                    picked_direction = Some(if backward { "backward" } else { "forward" });
                    let was_inserting = was_start == was_end && was_start < max;
                    if backward && !was_inserting {
                        offset = 1;
                    }
                }
                picked = Some((caret - offset, caret - offset + 1));
            }
        }
        if let Some((from, to)) = picked.filter(|(from, to)| from != to) {
            let _ = input.set_selection_range_with_direction(
                from,
                to,
                picked_direction.unwrap_or("none"),
            );
        }

        let (start, end) = picked.map_or((start, end), |(from, to)| (Some(from), Some(to)));
        otp.mirror.set(start.zip(end));
        otp.previous.set_value(Selection { start, end });
    }

    // what the page last saw of the input, after the browser had a moment to settle it
    fn resync(self) {
        let Some(otp) = self.try_otp() else {
            return;
        };
        let Some(input) = otp.input.get_untracked() else {
            return;
        };
        // an input event clears the :autofill look the browser left on it
        if let Ok(event) = web_sys::Event::new("input") {
            let _ = input.dispatch_event(&event);
        }
        if let (Ok(Some(start)), Ok(Some(end))) = (input.selection_start(), input.selection_end()) {
            otp.mirror.set(Some((start, end)));
            otp.previous.set_value(Selection {
                start: Some(start),
                end: Some(end),
            });
        }
    }
}

fn last(max_length: usize) -> u32 {
    u32::try_from(max_length.saturating_sub(1)).unwrap_or(u32::MAX)
}

fn is_ios() -> bool {
    web_sys::css::supports_with_value("-webkit-touch-callout", "none").unwrap_or(false)
}

// one input over the slots: the slots draw its value and its selection
#[component(element = "div", skip(style))]
pub fn OtpRoot(
    max_length: usize,
    #[prop(optional_no_strip, into)] value: Option<Signal<String>>,
    #[prop(optional_no_strip, into)] default_value: Option<String>,
    #[prop(optional_no_strip, into)] on_value_change: Option<Callback<(String, ChangeReason)>>,
    #[prop(optional_no_strip, into)] on_complete: Option<Callback<String>>,
    #[prop(optional_no_strip, into)] pattern: Option<Signal<String>>,
    #[prop(into, default = InputMode::Numeric.into())] input_mode: Signal<InputMode>,
    #[prop(optional_no_strip, into)] placeholder: Option<Signal<String>>,
    #[prop(optional_no_strip, into)] autocomplete: Option<Signal<String>>,
    #[prop(optional_no_strip, into)] paste_transformer: Option<Callback<String, String>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let otp = Otp {
        value: Controlled::new(value, default_value.unwrap_or_default(), on_value_change),
        max_length,
        pattern,
        input_mode,
        placeholder,
        autocomplete,
        disabled,
        on_complete,
        paste_transformer,
        input: NodeRef::new(),
        focused: RwSignal::new(false),
        hovering: RwSignal::new(false),
        mirror: RwSignal::new(None),
        previous: StoredValue::new(Selection::default()),
    };
    let context = OtpContext(StoredValue::new(otp));
    let root_height = RwSignal::new(None::<i32>);
    let initial = otp.value.get_untracked();

    Effect::new(move |_| {
        let Some(input) = otp.input.get() else {
            return;
        };
        // the browser may have restored a value of its own
        if input.value() != initial {
            otp.value.set(input.value(), ChangeReason::None);
        }
        otp.previous.set_value(Selection {
            start: input.selection_start().ok().flatten(),
            end: input.selection_end().ok().flatten(),
        });

        let listener = Closure::<dyn Fn(web_sys::Event)>::new(move |_| context.selection_changed());
        let options = web_sys::AddEventListenerOptions::new();
        options.set_capture(true);
        let _ = document().add_event_listener_with_callback_and_add_event_listener_options(
            "selectionchange",
            listener.as_ref().unchecked_ref(),
            &options,
        );
        let held = StoredValue::new_local(listener);
        on_cleanup(move || {
            held.try_with_value(|listener| {
                let _ = document().remove_event_listener_with_callback_and_bool(
                    "selectionchange",
                    listener.as_ref().unchecked_ref(),
                    true,
                );
            });
        });

        context.selection_changed();
        if document()
            .active_element()
            .is_some_and(|active| active == **input)
        {
            otp.focused.set(true);
        }

        let measured = input.clone();
        let measure = move || root_height.set(Some(measured.client_height()));
        measure();
        observe_resize(&input, measure);
    });

    Effect::new(move |_| {
        otp.value.track();
        otp.focused.track();
        for delay in [0, 10, 50] {
            set_timeout(move || context.resync(), Duration::from_millis(delay));
        }
    });

    let style = move || {
        let cursor = if disabled.get() { "default" } else { "text" };
        let height = root_height
            .get()
            .map(|height| format!(" --root-height: {height}px;"))
            .unwrap_or_default();
        format!(
            "position: relative; cursor: {cursor}; user-select: none; -webkit-user-select: none; pointer-events: none;{height}"
        )
    };

    view! {
        <noscript inner_html=NOSCRIPT_CSS_FALLBACK></noscript>
        <div {..attrs} data-input-otp-container="true" style=style>
            <Provider value=context>{children.map(|children| children())}</Provider>
        </div>
    }
}

// for the <input> inside an OtpRoot, in a box styled with OTP_INPUT_WRAPPER_STYLE
pub fn otp_input_attributes() -> impl Attribute {
    let context = use_otp();
    let otp = context
        .try_otp()
        .expect("otp_input_attributes runs inside an OtpRoot");
    let value = otp.value;
    let mirror = otp.mirror;
    let attributes = view! {
        <{..}
            autocomplete=move || {
                otp.autocomplete.map_or_else(|| String::from("one-time-code"), |autocomplete| autocomplete.get())
            }
            data-input-otp="true"
            data-input-otp-placeholder-shown=move || value.with(String::is_empty).then_some("true")
            data-input-otp-mss=move || mirror.get().map(|(start, _)| start)
            data-input-otp-mse=move || mirror.get().map(|(_, end)| end)
            inputmode=keyword(otp.input_mode)
            pattern=move || otp.pattern.map(|pattern| pattern.get())
            aria-placeholder=move || otp.placeholder.map(|placeholder| placeholder.get())
            style=INPUT_STYLE
            maxlength=otp.max_length
            value=value.get_untracked()
            disabled=move || otp.disabled.get()
            on:input=move |event| context.changed(&event_target::<HtmlInputElement>(&event))
            on:paste=move |event: web_sys::ClipboardEvent| context.pasted(&event)
            on:mouseover=move |_| otp.hovering.set(true)
            on:mouseleave=move |_| otp.hovering.set(false)
            on:focus=move |_| context.focused()
            on:blur=move |_| otp.focused.set(false)
        />
    };
    let live: Signal<String> = value.into();
    (attributes, prop("value", live), node_ref(otp.input))
}
