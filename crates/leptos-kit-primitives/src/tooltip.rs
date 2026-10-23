use crate::anchor::{Placement, placement};
use crate::hover::Hover;
use crate::id::ElementId;
use crate::{Controlled, Native, NativeOpen, reason, use_id};
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos_kit_macros::component;
use web_sys::{Element, HtmlElement};

const OPEN_DELAY: u32 = 600;
const ARROW_PADDING: f64 = 5.0;

// base ui's delay group: while one tooltip is open, or just after, the next opens at once
#[derive(Clone, Copy)]
struct Group {
    delay: Signal<u32>,
    close_delay: Signal<u32>,
    timeout: Signal<u32>,
    open: StoredValue<Option<(usize, Callback<()>)>>,
    closed_at: StoredValue<f64>,
    next: StoredValue<usize>,
}

impl Group {
    fn warm(&self) -> bool {
        let recent = now() - self.closed_at.get_value() < f64::from(self.timeout.get_untracked());
        self.open.with_value(Option::is_some) || recent
    }

    fn opened(&self, key: usize, close: Callback<()>) {
        let previous = self
            .open
            .try_update_value(|open| open.replace((key, close)))
            .flatten();
        if let Some((other, close)) = previous
            && other != key
        {
            close.run(());
        }
    }

    fn closed(&self, key: usize) {
        if self
            .open
            .with_value(|open| open.is_some_and(|(open, _)| open == key))
        {
            self.open.set_value(None);
            self.closed_at.set_value(now());
        }
    }
}

fn now() -> f64 {
    web_sys::js_sys::Date::now()
}

#[leptos::component]
pub fn TooltipProvider(
    #[prop(optional_no_strip, into)] delay: Option<Signal<u32>>,
    #[prop(optional_no_strip, into)] close_delay: Option<Signal<u32>>,
    #[prop(into, default = 400.into())] timeout: Signal<u32>,
    children: Children,
) -> impl IntoView {
    let group = Group {
        delay: Signal::derive(move || delay.map_or(OPEN_DELAY, |delay| delay.get())),
        close_delay: Signal::derive(move || close_delay.map_or(0, |delay| delay.get())),
        timeout,
        open: StoredValue::new(None),
        closed_at: StoredValue::new(f64::NEG_INFINITY),
        next: StoredValue::new(0),
    };
    view! { <Provider value=group>{children()}</Provider> }
}

#[derive(Clone, Copy)]
struct TooltipContext {
    key: usize,
    id: ElementId<html::Div>,
    native: NativeOpen<html::Div>,
    positioner: NodeRef<html::Div>,
    popup: NodeRef<html::Div>,
    trigger: NodeRef<html::Button>,
    disabled: Signal<bool>,
    hoverable: Signal<bool>,
    hover: Hover,
    group: Option<Group>,
}

impl TooltipContext {
    fn open(&self) -> bool {
        self.native.state().get()
    }

    fn anchor(&self) -> String {
        format!("--{}", self.id.get())
    }

    fn set(self, open: bool, reason: &'static str) {
        if open == self.native.state().get_untracked() || (open && self.disabled.get_untracked()) {
            return;
        }
        if let Some(group) = self.group {
            if open {
                group.opened(self.key, self.closer(reason::NONE));
            } else {
                group.closed(self.key);
            }
        }
        self.native.state().set(open, reason);
    }

    fn opener(self, reason: &'static str) -> Callback<()> {
        Callback::new(move |_| self.set(true, reason))
    }

    fn closer(self, reason: &'static str) -> Callback<()> {
        Callback::new(move |_| self.set(false, reason))
    }

    // the two boxes the pointer may cross between while the tooltip stays
    fn boxes(&self) -> Option<[Element; 2]> {
        let trigger = self.trigger.get_untracked()?;
        let popup = self.popup.get_untracked()?;
        Some([trigger.into(), popup.into()])
    }
}

// a manual popover: hover and focus open it, nothing the browser does closes it
#[leptos::component]
pub fn TooltipRoot(
    #[prop(optional, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, &'static str)>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] disable_hoverable_popup: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let positioner = NodeRef::new();
    let popup = NodeRef::new();
    let state = Controlled::new(open, default_open, on_open_change);
    let group = use_context::<Group>();
    let key = group.map_or(0, |group| {
        group
            .next
            .try_update_value(|next| {
                *next += 1;
                *next
            })
            .unwrap_or_default()
    });
    let context = TooltipContext {
        key,
        id: ElementId::new(positioner, use_id()),
        native: NativeOpen::new(Native::Popover, positioner, state, Some(popup)),
        positioner,
        popup,
        trigger: NodeRef::new(),
        disabled,
        hoverable: Signal::derive(move || !disable_hoverable_popup.get()),
        hover: Hover::new(),
        group,
    };
    // a group keeps one open, so of those open from the start only the last stays, as base ui's
    if let Some(group) = group
        && state.get_untracked()
    {
        group.opened(key, context.closer(reason::NONE));
    }
    Effect::new(move || {
        if disabled.get() && state.get_untracked() {
            context.set(false, reason::DISABLED);
        }
    });
    // esc closes it wherever the focus is, so the listener lives as long as it's open
    let escape = StoredValue::new(None::<WindowListenerHandle>);
    Effect::new(move || {
        if let Some(Some(handle)) = escape.try_update_value(Option::take) {
            handle.remove();
        }
        if state.get() {
            let handle = window_event_listener(ev::keydown, move |event| {
                if event.key() == "Escape" {
                    context.set(false, reason::ESCAPE_KEY);
                }
            });
            escape.set_value(Some(handle));
        }
    });
    on_cleanup(move || {
        if let Some(Some(handle)) = escape.try_update_value(Option::take) {
            handle.remove();
        }
        if let Some(group) = group {
            group.closed(key);
        }
    });
    // components share their parent's owner, a sibling root would overwrite the context
    view! { <Provider value=context>{children()}</Provider> }
}

#[component(element = "button", skip(r#type))]
pub fn TooltipTrigger(
    #[prop(optional_no_strip, into)] delay: Option<Signal<u32>>,
    #[prop(optional_no_strip, into)] close_delay: Option<Signal<u32>>,
    #[prop(into, default = true.into())] close_on_click: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<TooltipContext>();
    let open = move || context.open();
    let open_delay = move || {
        if context.group.is_some_and(|group| group.warm()) {
            return 0;
        }
        delay
            .map(|delay| delay.get_untracked())
            .or_else(|| context.group.map(|group| group.delay.get_untracked()))
            .unwrap_or(OPEN_DELAY)
    };
    let close_delay = move || {
        close_delay
            .map(|delay| delay.get_untracked())
            .or_else(|| context.group.map(|group| group.close_delay.get_untracked()))
            .unwrap_or(0)
    };
    let mouse = |event: &ev::PointerEvent| event.pointer_type() == "mouse";
    let pressed = move || {
        if !close_on_click.get_untracked() {
            return;
        }
        context.hover.cancel();
        context.set(false, reason::TRIGGER_PRESS);
    };

    view! {
        <button
            {..attrs}
            node_ref=context.trigger
            type="button"
            style:anchor-name=context.anchor()
            data-popup-open=move || open().then_some("")
            data-trigger-disabled=move || context.disabled.get().then_some("")
            on:pointerenter=move |event: ev::PointerEvent| {
                if mouse(&event) && !context.disabled.get_untracked() {
                    context.hover.enter(open_delay(), context.opener(reason::TRIGGER_HOVER));
                }
            }
            on:pointerleave=move |event: ev::PointerEvent| {
                if !mouse(&event) {
                    return;
                }
                let close = context.closer(reason::TRIGGER_HOVER);
                match context.boxes() {
                    Some(boxes) if context.hoverable.get_untracked() && context.open() => {
                        let from = (f64::from(event.client_x()), f64::from(event.client_y()));
                        context.hover.cross(from, boxes, close_delay(), close);
                    }
                    _ => context.hover.leave(close_delay(), close),
                }
            }
            on:focus=move |event: ev::FocusEvent| {
                // focus from the keyboard only, as base ui's
                let visible = event_target::<HtmlElement>(&event)
                    .matches(":focus-visible")
                    .unwrap_or(false);
                if visible {
                    context.hover.cancel();
                    context.set(true, reason::TRIGGER_FOCUS);
                }
            }
            on:blur=move |_| {
                context.hover.cancel();
                context.set(false, reason::TRIGGER_FOCUS);
            }
            on:pointerdown=move |_| pressed()
            on:click=move |_| pressed()
        >
            {children.map(|children| children())}
        </button>
    }
}

#[component(element = "div", skip(id, style, role))]
pub fn TooltipPositioner(
    #[prop(into, default = "top".into())] side: Signal<&'static str>,
    #[prop(into, default = "center".into())] align: Signal<&'static str>,
    #[prop(optional, into)] side_offset: Signal<f64>,
    #[prop(optional, into)] align_offset: Signal<f64>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<TooltipContext>();
    let placed = Placement::new(side, align);
    let fit = Fit {
        arrow: NodeRef::new(),
        offset: RwSignal::new(None),
        snap: RwSignal::new((0.0, 0.0)),
    };
    let open = move || context.open();
    let style = move || {
        #[cfg(debug_assertions)]
        crate::anchor::check("TooltipPositioner", side.get(), align.get());
        placement(
            &context.anchor(),
            side.get(),
            align.get(),
            side_offset.get(),
            align_offset.get(),
        )
    };
    // show_popover runs in the root's effect, so the boxes are in place a microtask later
    Effect::new(move || {
        if context.native.state().get() {
            queue_microtask(move || settle(context, placed, fit));
        } else {
            placed.flipped.try_set(false);
        }
    });
    view! {
        <div
            {..attrs}
            node_ref=context.positioner
            id=context.id.get()
            popover="manual"
            role="presentation"
            style=style
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-side=move || placed.side()
            data-align=align
            on:toggle=move |_| context.native.sync(reason::NONE)
        >
            <Provider value=placed>
                <Provider value=fit>{children.map(|children| children())}</Provider>
            </Provider>
        </div>
    }
}

// what settling finds: the nudge that puts the popup on whole pixels, and where the arrow points
#[derive(Clone, Copy)]
struct Fit {
    arrow: NodeRef<html::Div>,
    offset: RwSignal<Option<(f64, bool)>>,
    snap: RwSignal<(f64, f64)>,
}

fn settle(context: TooltipContext, placed: Placement, fit: Fit) {
    let (Some(Some(trigger)), Some(Some(positioner))) = (
        context.trigger.try_get_untracked(),
        context.positioner.try_get_untracked(),
    ) else {
        return;
    };
    placed.settle(&trigger, &positioner);
    // the popup may still be scaled by its opening animation, the positioner and the arrow's layout size aren't
    let (t, p) = (
        trigger.get_bounding_client_rect(),
        positioner.get_bounding_client_rect(),
    );
    // floating-ui puts a popup on device pixels, css centring can leave it between two
    let ratio = window().device_pixel_ratio();
    let (left, top) = (p.left(), p.top());
    let (snapped_left, snapped_top) = (
        (left * ratio).round() / ratio,
        (top * ratio).round() / ratio,
    );
    fit.snap.try_set((snapped_left - left, snapped_top - top));
    let Some(Some(element)) = fit.arrow.try_get_untracked() else {
        return;
    };
    let Some(Some(popup)) = context.popup.try_get_untracked() else {
        return;
    };
    // floating-ui's arrow: centred on the popup's whole-pixel client size, moved by how far the popup sits off the trigger
    let vertical = matches!(placed.side.get_untracked(), "top" | "bottom");
    let (shift, client, size) = if vertical {
        (
            (t.left() + t.right()) / 2.0 - (left + p.width() / 2.0),
            f64::from(popup.client_width()),
            f64::from(element.offset_width()),
        )
    } else {
        (
            (t.top() + t.bottom()) / 2.0 - (top + p.height() / 2.0),
            f64::from(popup.client_height()),
            f64::from(element.offset_height()),
        )
    };
    let padding = ARROW_PADDING.min(client / 2.0 - size / 2.0 - 1.0);
    let centre = client / 2.0 - size / 2.0 + shift;
    let offset = centre.max(padding).min(client - size - padding);
    fit.offset
        .try_set(Some((offset, (offset - centre).abs() > 0.01)));
}

#[component(element = "div")]
pub fn TooltipPopup(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<TooltipContext>();
    let placed = expect_context::<Placement>();
    let fit = expect_context::<Fit>();
    let open = move || context.open();
    let nudge = move |pick: fn((f64, f64)) -> f64| move || format!("{}px", pick(fit.snap.get()));
    let close_delay = move || {
        context
            .group
            .map_or(0, |group| group.close_delay.get_untracked())
    };
    view! {
        <div
            {..attrs}
            node_ref=context.popup
            // a layout nudge, a fraction of a pixel in a translate would blur it
            style:position="relative"
            style:left=nudge(|(x, _)| x)
            style:top=nudge(|(_, y)| y)
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-side=move || placed.side()
            data-align=placed.align
            on:pointerenter=move |_| {
                if context.hoverable.get_untracked() {
                    context.hover.hold();
                }
            }
            on:pointerleave=move |event: ev::PointerEvent| {
                if !context.hoverable.get_untracked() || event.pointer_type() != "mouse" {
                    return;
                }
                let close = context.closer(reason::TRIGGER_HOVER);
                if let Some(boxes) = context.boxes() {
                    let from = (f64::from(event.client_x()), f64::from(event.client_y()));
                    context.hover.cross(from, boxes, close_delay(), close);
                }
            }
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn TooltipArrow(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<TooltipContext>();
    let placed = expect_context::<Placement>();
    let fit = expect_context::<Fit>();
    let open = move || context.open();
    let vertical = move || matches!(placed.side.get(), "top" | "bottom");
    let at = move |along: bool| {
        move || {
            let (offset, _) = fit.offset.get()?;
            (vertical() == along).then(|| format!("{offset}px"))
        }
    };
    view! {
        <div
            {..attrs}
            node_ref=fit.arrow
            aria-hidden="true"
            style:position="absolute"
            style:left=at(true)
            style:top=at(false)
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-side=move || placed.side()
            data-align=placed.align
            data-uncentered=move || fit.offset.get().is_some_and(|(_, off)| off).then_some("")
        >
            {children.map(|children| children())}
        </div>
    }
}
