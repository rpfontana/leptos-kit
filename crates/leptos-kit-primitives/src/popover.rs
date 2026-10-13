use crate::button::button_attributes;
use crate::{Controlled, Native, NativeOpen, reason, use_id};
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[derive(Clone, Copy)]
struct PopoverContext {
    id: StoredValue<String>,
    native: NativeOpen<html::Div>,
    positioner: NodeRef<html::Div>,
    popup: NodeRef<html::Div>,
    trigger: NodeRef<html::Button>,
    title: RwSignal<Option<String>>,
    description: RwSignal<Option<String>>,
}

// what the positioner tells the popup inside it
#[derive(Clone, Copy)]
struct Placement {
    side: Signal<&'static str>,
    align: Signal<&'static str>,
    flipped: RwSignal<bool>,
}

impl Placement {
    fn side(&self) -> &'static str {
        let side = self.side.get();
        if self.flipped.get() {
            opposite(side)
        } else {
            side
        }
    }
}

impl PopoverContext {
    fn open(&self) -> bool {
        self.native.state().get()
    }

    fn popup_id(&self) -> String {
        format!("{}-popup", self.id.get_value())
    }

    fn anchor(&self) -> String {
        format!("--{}", self.id.get_value())
    }
}

#[leptos::component]
pub fn PopoverRoot(
    #[prop(optional, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, &'static str)>>,
    children: Children,
) -> impl IntoView {
    let positioner = NodeRef::new();
    let popup = NodeRef::new();
    let state = Controlled::new(open, default_open, on_open_change);
    provide_context(PopoverContext {
        id: StoredValue::new(use_id()),
        native: NativeOpen::new(Native::Popover, positioner, state, Some(popup)),
        positioner,
        popup,
        trigger: NodeRef::new(),
        title: RwSignal::new(None),
        description: RwSignal::new(None),
    });
    children()
}

#[component(element = "button", skip(r#type, popovertarget, popovertargetaction))]
pub fn PopoverTrigger(
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<PopoverContext>();
    let disabled = move || disabled.is_some_and(|disabled| disabled.get());
    let open = move || context.open();

    view! {
        <button
            {..attrs}
            {..button_attributes(disabled, || false, tabindex)}
            node_ref=context.trigger
            type="button"
            style:anchor-name=context.anchor()
            popovertarget=context.id.get_value()
            aria-haspopup="dialog"
            aria-expanded=move || open().to_string()
            aria-controls=move || open().then(|| context.popup_id())
            data-popup-open=move || open().then_some("")
            data-pressed=move || open().then_some("")
            on:click=move |event: ev::MouseEvent| {
                // a native close would skip the exit animation
                if context.native.state().get_untracked() {
                    event.prevent_default();
                    context.native.state().set(false, reason::TRIGGER_PRESS);
                } else {
                    context.native.expect(reason::TRIGGER_PRESS);
                }
            }
        >
            {children.map(|children| children())}
        </button>
    }
}

#[component(element = "div", skip(id, style, role))]
pub fn PopoverPositioner(
    #[prop(into, default = "bottom".into())] side: Signal<&'static str>,
    #[prop(into, default = "center".into())] align: Signal<&'static str>,
    #[prop(optional, into)] side_offset: Signal<f64>,
    #[prop(optional, into)] align_offset: Signal<f64>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<PopoverContext>();
    let placed = Placement {
        side,
        align,
        flipped: RwSignal::new(false),
    };
    provide_context(placed);
    let open = move || context.open();
    let anchor = context.anchor();
    let style = move || {
        #[cfg(debug_assertions)]
        check(side.get(), align.get());
        placement(
            &anchor,
            side.get(),
            align.get(),
            side_offset.get(),
            align_offset.get(),
        )
    };

    view! {
        <div
            {..attrs}
            node_ref=context.positioner
            id=context.id.get_value()
            popover="auto"
            role="presentation"
            style=style
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-side=move || placed.side()
            data-align=align
            on:toggle=move |_| {
                context.native.sync(reason::OUTSIDE_PRESS);
                match context.native.state().try_get_untracked() {
                    Some(true) => request_animation_frame(move || settle(context, placed)),
                    Some(false) => {
                        placed.flipped.try_set(false);
                    }
                    None => {}
                }
            }
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div", skip(id, role))]
pub fn PopoverPopup(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<PopoverContext>();
    let placed = expect_context::<Placement>();
    let open = move || context.open();
    view! {
        <div
            {..attrs}
            node_ref=context.popup
            id=context.popup_id()
            role="dialog"
            tabindex="-1"
            aria-labelledby=move || context.title.get()
            aria-describedby=move || context.description.get()
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-side=move || placed.side()
            data-align=placed.align
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "button", skip(r#type, popovertarget, popovertargetaction))]
pub fn PopoverClose(
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<PopoverContext>();
    let disabled = move || disabled.is_some_and(|disabled| disabled.get());
    view! {
        <button
            {..attrs}
            {..button_attributes(disabled, || false, tabindex)}
            type="button"
            popovertarget=context.id.get_value()
            popovertargetaction="hide"
            on:click=move |event: ev::MouseEvent| {
                event.prevent_default();
                context.native.state().set(false, reason::CLOSE_PRESS);
            }
        >
            {children.map(|children| children())}
        </button>
    }
}

#[component(element = "h2")]
pub fn PopoverTitle(
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<PopoverContext>();
    let id = id.unwrap_or_else(|| format!("{}-title", context.id.get_value()));
    context.title.set(Some(id.clone()));
    on_cleanup(move || context.title.set(None));
    view! {
        <h2 {..attrs} id=id>
            {children.map(|children| children())}
        </h2>
    }
}

#[component(element = "p")]
pub fn PopoverDescription(
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<PopoverContext>();
    let id = id.unwrap_or_else(|| format!("{}-description", context.id.get_value()));
    context.description.set(Some(id.clone()));
    on_cleanup(move || context.description.set(None));
    view! {
        <p {..attrs} id=id>
            {children.map(|children| children())}
        </p>
    }
}

fn placement(anchor: &str, side: &str, align: &str, side_offset: f64, align_offset: f64) -> String {
    let (block, span_start, span_end, fallback) = match side {
        "top" => ("top", "span-right", "span-left", "flip-block"),
        "left" => ("left", "span-bottom", "span-top", "flip-inline"),
        "right" => ("right", "span-bottom", "span-top", "flip-inline"),
        "inline-start" => (
            "inline-start",
            "span-block-end",
            "span-block-start",
            "flip-inline",
        ),
        "inline-end" => (
            "inline-end",
            "span-block-end",
            "span-block-start",
            "flip-inline",
        ),
        _ => ("bottom", "span-right", "span-left", "flip-block"),
    };
    let area = match align {
        "start" => format!("{block} {span_start}"),
        "end" => format!("{block} {span_end}"),
        _ => block.to_string(),
    };
    let facing = match side {
        "top" => "margin-bottom",
        "left" => "margin-right",
        "right" => "margin-left",
        "inline-start" => "margin-inline-end",
        "inline-end" => "margin-inline-start",
        _ => "margin-top",
    };
    let along = match (side, align) {
        ("left" | "right" | "inline-start" | "inline-end", "end") => "margin-bottom",
        ("left" | "right" | "inline-start" | "inline-end", _) => "margin-top",
        (_, "end") => "margin-right",
        _ => "margin-left",
    };
    let origin = match side {
        "top" => "bottom",
        "left" => "right",
        "right" => "left",
        "inline-start" => "right",
        "inline-end" => "left",
        _ => "top",
    };
    format!(
        "inset:auto;margin:0;padding:0;border:0;background:transparent;color:inherit;overflow:visible;\
         position-anchor:{anchor};position-area:{area};position-try-fallbacks:{fallback};\
         {facing}:{side_offset}px;{along}:{align_offset}px;--transform-origin:{origin}"
    )
}

// css doesn't say which fallback won, so read the side off the boxes
fn settle(context: PopoverContext, placed: Placement) {
    let (Some(Some(trigger)), Some(Some(popup)), Some(side)) = (
        context.trigger.try_get_untracked(),
        context.positioner.try_get_untracked(),
        placed.side.try_get_untracked(),
    ) else {
        return;
    };
    let (t, p) = (
        trigger.get_bounding_client_rect(),
        popup.get_bounding_client_rect(),
    );
    let flipped = match side {
        "top" => p.top() >= t.bottom() - 1.0,
        "bottom" => p.bottom() <= t.top() + 1.0,
        "left" | "inline-start" => p.left() >= t.right() - 1.0,
        "right" | "inline-end" => p.right() <= t.left() + 1.0,
        _ => false,
    };
    placed.flipped.try_set(flipped);
}

fn opposite(side: &'static str) -> &'static str {
    match side {
        "top" => "bottom",
        "bottom" => "top",
        "left" => "right",
        "right" => "left",
        "inline-start" => "inline-end",
        _ => "inline-start",
    }
}

#[cfg(debug_assertions)]
fn check(side: &str, align: &str) {
    const SIDES: &[&str] = &[
        "top",
        "bottom",
        "left",
        "right",
        "inline-start",
        "inline-end",
    ];
    if !SIDES.contains(&side) {
        leptos::logging::warn!(
            "PopoverPositioner: unknown side `{side}`, expected one of {SIDES:?}"
        );
    }
    if !["start", "center", "end"].contains(&align) {
        leptos::logging::warn!(
            "PopoverPositioner: unknown align `{align}`, expected start, center or end"
        );
    }
}
