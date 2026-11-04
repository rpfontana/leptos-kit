use super::utils::classes;
use leptos::prelude::*;
use leptos_kit::icons::{Icon, semantic};
use leptos_kit::macros::component;
use leptos_kit::primitives::{self, ChangeReason};

pub use primitives::{
    InputMode, REGEXP_ONLY_CHARS, REGEXP_ONLY_DIGITS, REGEXP_ONLY_DIGITS_AND_CHARS,
};

#[component(element = "input", skip(maxlength, style))]
pub fn InputOtp(
    max_length: usize,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] container_class: Option<Signal<String>>,
    #[prop(optional, into)] value: Option<Signal<String>>,
    #[prop(optional, into)] default_value: Option<String>,
    #[prop(optional, into)] on_value_change: Option<Callback<(String, ChangeReason)>>,
    #[prop(optional, into)] on_complete: Option<Callback<String>>,
    #[prop(optional, into)] pattern: Option<Signal<String>>,
    #[prop(into, default = InputMode::Numeric.into())] input_mode: Signal<InputMode>,
    #[prop(optional, into)] placeholder: Option<Signal<String>>,
    #[prop(optional, into)] autocomplete: Option<Signal<String>>,
    #[prop(optional, into)] paste_transformer: Option<Callback<String, String>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::OtpRoot
            max_length=max_length
            value=value
            default_value=default_value
            on_value_change=on_value_change
            on_complete=on_complete
            pattern=pattern
            input_mode=input_mode
            placeholder=placeholder
            autocomplete=autocomplete
            paste_transformer=paste_transformer
            disabled=disabled
            class=classes("cn-input-otp", container_class)
        >
            {children.map(|children| children())}
            <div style=primitives::OTP_INPUT_WRAPPER_STYLE>
                <input
                    {..attrs}
                    {..primitives::otp_input_attributes()}
                    class=classes("cn-input-otp-input", class)
                    data-slot="input-otp"
                    spellcheck="false"
                />
            </div>
        </primitives::OtpRoot>
    }
}

#[component(element = "div")]
pub fn InputOtpGroup(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div data-slot="input-otp-group" class=classes("cn-input-otp-group", class) {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn InputOtpSlot(
    index: usize,
    #[prop(optional, into)] class: Option<Signal<String>>,
) -> impl IntoView {
    let otp = primitives::use_otp();
    let slot = Memo::new(move |_| otp.slot(index));
    view! {
        <div
            data-slot="input-otp-slot"
            data-active=move || if slot.get().is_active { "true" } else { "false" }
            class=classes("cn-input-otp-slot", class)
            {..attrs}
        >
            {move || slot.get().char.map(String::from)}
            {move || {
                slot.get()
                    .has_fake_caret
                    .then(|| {
                        view! {
                            <div class="cn-input-otp-caret">
                                <div class="cn-input-otp-caret-line"></div>
                            </div>
                        }
                    })
            }}
        </div>
    }
}

#[component(element = "div", skip(role))]
pub fn InputOtpSeparator(#[prop(optional, into)] class: Option<Signal<String>>) -> impl IntoView {
    view! {
        <div
            data-slot="input-otp-separator"
            class=classes("cn-input-otp-separator", class)
            role="separator"
            {..attrs}
        >
            <Icon icon=semantic::INPUT_OTP_MINUS />
        </div>
    }
}
