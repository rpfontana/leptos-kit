use crate::use_direction;
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::tachys::html::event::on;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;
use leptos_kit_macros::component;
use std::time::Duration;
use web_sys::{HtmlElement, MouseEvent};

const SCROLL_TIMEOUT: Duration = Duration::from_millis(500);
const MIN_THUMB_SIZE: f64 = 16.0;

// the viewport along one axis: what scrolls, what shows, how far it has gone
#[derive(Clone, Copy)]
struct Extent {
    content: f64,
    size: f64,
    scroll: f64,
}

impl Extent {
    fn of(viewport: &HtmlElement, vertical: bool) -> Self {
        let [content, size, scroll] = if vertical {
            [
                viewport.scroll_height(),
                viewport.client_height(),
                viewport.scroll_top(),
            ]
        } else {
            [
                viewport.scroll_width(),
                viewport.client_width(),
                viewport.scroll_left(),
            ]
        };
        Self {
            content: content.into(),
            size: size.into(),
            scroll: scroll.into(),
        }
    }

    fn range(self) -> f64 {
        (self.content - self.size).max(0.0)
    }
}

fn scroll_to(viewport: &HtmlElement, vertical: bool, value: f64) {
    if vertical {
        viewport.set_scroll_top(value as i32);
    } else {
        viewport.set_scroll_left(value as i32);
    }
}

fn along(element: &HtmlElement, vertical: bool) -> f64 {
    f64::from(if vertical {
        element.offset_height()
    } else {
        element.offset_width()
    })
}

fn client(event: &MouseEvent, vertical: bool) -> f64 {
    f64::from(if vertical {
        event.client_y()
    } else {
        event.client_x()
    })
}

// how far the thumb can travel in its scrollbar
fn travel(scrollbar: &HtmlElement, thumb: &HtmlElement, thumb_size: f64, vertical: bool) -> f64 {
    along(scrollbar, vertical)
        - thumb_size
        - offset(scrollbar, "padding", vertical)
        - offset(thumb, "margin", vertical)
}

#[derive(Clone, Copy, Default)]
struct Drag {
    pointer: Option<i32>,
    vertical: bool,
    start: f64,
    scroll: f64,
}

// the pairs are x then y, indexed by `vertical as usize`
#[derive(Clone, Copy)]
struct ScrollAreaContext {
    viewport: NodeRef<html::Div>,
    scrollbars: [NodeRef<html::Div>; 2],
    thumbs: [NodeRef<html::Div>; 2],
    corner: NodeRef<html::Div>,
    hovering: RwSignal<bool>,
    scrolling: [RwSignal<bool>; 2],
    measured: RwSignal<bool>,
    hidden: RwSignal<[bool; 2]>,
    // x start, x end, y start, y end
    edges: RwSignal<[bool; 4]>,
    corner_size: RwSignal<[f64; 2]>,
    thumb_size: RwSignal<[f64; 2]>,
    threshold: Signal<f64>,
    direction: Signal<&'static str>,
    drag: StoredValue<Drag>,
    position: StoredValue<[f64; 2]>,
    timeouts: StoredValue<[Option<TimeoutHandle>; 2]>,
    touch: StoredValue<bool>,
    snap: StoredValue<Option<String>>,
}

impl ScrollAreaContext {
    fn rtl(self) -> bool {
        self.direction.get_untracked() == "rtl"
    }

    fn scrolling_any(self) -> impl Fn() -> bool + Copy + Send + Sync + 'static {
        move || self.scrolling[0].get() || self.scrolling[1].get()
    }

    fn start_scrolling(self, vertical: bool) {
        let axis = vertical as usize;
        change(self.scrolling[axis], true);
        let timeout = set_timeout_with_handle(
            move || {
                self.scrolling[axis].try_set(false);
            },
            SCROLL_TIMEOUT,
        )
        .ok();
        let previous = self
            .timeouts
            .try_update_value(|all| std::mem::replace(&mut all[axis], timeout));
        if let Some(Some(previous)) = previous {
            previous.clear();
        }
    }

    fn scrolled(self, viewport: &HtmlElement) {
        let now = [false, true].map(|vertical| Extent::of(viewport, vertical).scroll);
        let before = self.position.get_value();
        self.position.set_value(now);
        for vertical in [true, false] {
            if now[vertical as usize] != before[vertical as usize] {
                self.start_scrolling(vertical);
            }
        }
    }

    // a drag or a track click would fight scroll snapping
    fn disable_snap(self) {
        let Some(viewport) = self.viewport.get_untracked() else {
            return;
        };
        if self.snap.with_value(Option::is_none) {
            let style = HtmlElement::style(&viewport);
            let snap = style
                .get_property_value("scroll-snap-type")
                .unwrap_or_default();
            self.snap.set_value(Some(snap));
            let _ = style.set_property("scroll-snap-type", "none");
        }
    }

    fn press(self, event: &ev::PointerEvent, vertical: bool) {
        if event.button() != 0 {
            return;
        }
        let drag = self.drag.get_value();
        let thumb = self.thumbs[drag.vertical as usize].get_untracked();
        if let (Some(pointer), Some(thumb)) = (drag.pointer, thumb)
            && thumb.has_pointer_capture(pointer)
        {
            return;
        }
        let Some(viewport) = self.viewport.get_untracked() else {
            return;
        };
        self.drag.set_value(Drag {
            pointer: Some(event.pointer_id()),
            vertical,
            start: client(event, vertical),
            scroll: Extent::of(&viewport, vertical).scroll,
        });
        self.disable_snap();
        if let Some(thumb) = self.thumbs[vertical as usize].get_untracked() {
            let _ = thumb.set_pointer_capture(event.pointer_id());
        }
    }

    fn release(self, event: &ev::PointerEvent) {
        let drag = self.drag.get_value();
        if drag.pointer != Some(event.pointer_id()) {
            return;
        }
        self.drag.update_value(|drag| drag.pointer = None);
        change(self.scrolling[drag.vertical as usize], false);
        if let Some(snap) = self.snap.try_update_value(Option::take).flatten()
            && let Some(viewport) = self.viewport.get_untracked()
        {
            let _ = HtmlElement::style(&viewport).set_property("scroll-snap-type", &snap);
        }
        if let Some(thumb) = self.thumbs[drag.vertical as usize].get_untracked()
            && thumb.has_pointer_capture(event.pointer_id())
        {
            let _ = thumb.release_pointer_capture(event.pointer_id());
        }
    }

    fn drag(self, event: &ev::PointerEvent) {
        let drag = self.drag.get_value();
        if drag.pointer != Some(event.pointer_id()) {
            return;
        }
        // the button came up outside the page
        if event.buttons() & 1 == 0 {
            self.release(event);
            return;
        }
        let vertical = drag.vertical;
        let axis = vertical as usize;
        let (Some(viewport), Some(thumb), Some(scrollbar)) = (
            self.viewport.get_untracked(),
            self.thumbs[axis].get_untracked(),
            self.scrollbars[axis].get_untracked(),
        ) else {
            return;
        };
        let travel = travel(&scrollbar, &thumb, along(&thumb, vertical), vertical);
        let moved = client(event, vertical) - drag.start;
        let ratio = if travel <= 0.0 { 0.0 } else { moved / travel };
        let range = Extent::of(&viewport, vertical).range();
        scroll_to(&viewport, vertical, drag.scroll + ratio * range);
        event.prevent_default();
        self.start_scrolling(vertical);
    }

    // base ui's computeThumbPosition: thumb sizes and places, the corner, which axes overflow
    fn measure(self) {
        let Some(Some(viewport)) = self.viewport.try_get_untracked() else {
            return;
        };
        change(self.measured, true);
        let rtl = self.rtl();
        let extents = [false, true].map(|vertical| Extent::of(&viewport, vertical));
        if extents.iter().any(|extent| extent.content == 0.0) {
            return;
        }
        let hidden = extents.map(|extent| extent.size >= extent.content);
        let parts = [0, 1].map(|axis| {
            let scrollbar = self.scrollbars[axis]
                .get_untracked()
                .filter(|bar| !bar.hidden())?;
            let thumb = self.thumbs[axis].get_untracked().map(HtmlElement::from);
            Some((HtmlElement::from(scrollbar), thumb))
        });
        // each scrollbar's thickness, where the other one meets it
        let corner = if hidden[0] || hidden[1] {
            [0.0, 0.0]
        } else {
            [(1, false), (0, true)].map(|(axis, vertical)| {
                parts[axis]
                    .as_ref()
                    .map_or(0.0, |(bar, _)| along(bar, vertical))
            })
        };
        // until the corner has a size, the scrollbars still reach into it
        let unmeasured = self.corner_size.get_untracked() == [0.0, 0.0];
        let threshold = self.threshold.get_untracked();
        let mut edges = [false; 4];
        let mut sizes = [0.0; 2];
        for (axis, extent) in extents.into_iter().enumerate() {
            let vertical = axis == 1;
            let range = extent.range();
            let from_start = if rtl && !vertical {
                -extent.scroll
            } else {
                extent.scroll
            };
            let start = if hidden[axis] {
                0.0
            } else {
                normalize(from_start, range)
            };
            let end = if hidden[axis] { 0.0 } else { range - start };
            edges[axis * 2] = !hidden[axis] && start > threshold;
            edges[axis * 2 + 1] = !hidden[axis] && end > threshold;

            let (room, places) = match &parts[axis] {
                Some((bar, thumb)) => {
                    let inner = offset(bar, "padding", vertical)
                        + thumb
                            .as_ref()
                            .map_or(0.0, |thumb| offset(thumb, "margin", vertical));
                    let ideal = if hidden[axis] { 0.0 } else { extent.size } - inner;
                    let reach = along(bar, vertical) - if unmeasured { corner[axis] } else { 0.0 };
                    (ideal.min(reach), thumb.as_ref().map(|thumb| (bar, thumb)))
                }
                None => (if hidden[axis] { 0.0 } else { extent.size }, None),
            };
            let size = (room * extent.size / extent.content).max(MIN_THUMB_SIZE);
            sizes[axis] = size;
            if let Some((bar, thumb)) = places {
                let travel = travel(bar, thumb, size, vertical);
                let at = overscroll(
                    thumb,
                    vertical,
                    from_start,
                    range,
                    extent.content,
                    size,
                    travel,
                );
                let transform = if vertical {
                    format!("translate3d(0,{at}px,0)")
                } else {
                    format!("translate3d({}px,0,0)", if rtl { -at } else { at })
                };
                let _ = thumb.style().set_property("transform", &transform);
            }
            let style = HtmlElement::style(&viewport);
            let name = if vertical { "y" } else { "x" };
            let _ = style.set_property(
                &format!("--scroll-area-overflow-{name}-start"),
                &format!("{start}px"),
            );
            let _ = style.set_property(
                &format!("--scroll-area-overflow-{name}-end"),
                &format!("{end}px"),
            );
        }
        change(self.thumb_size, sizes);
        if self.corner.get_untracked().is_some() {
            change(self.corner_size, corner);
        }
        change(self.hidden, hidden);
        change(self.edges, edges);
    }
}

fn change<T: PartialEq + Send + Sync + 'static>(signal: RwSignal<T>, next: T) {
    signal.try_maybe_update(|value| {
        let changed = *value != next;
        if changed {
            *value = next;
        }
        (changed, ())
    });
}

// padding or margin along an axis, and base ui counts a horizontal thumb's start margin twice
fn offset(element: &HtmlElement, property: &str, vertical: bool) -> f64 {
    let Some(style) = window().get_computed_style(element).ok().flatten() else {
        return 0.0;
    };
    let side = if vertical { "block" } else { "inline" };
    let read = |edge: &str| {
        style
            .get_property_value(&format!("{property}-{side}-{edge}"))
            .ok()
            .and_then(|value| value.trim_end_matches("px").parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    if !vertical && property == "margin" {
        read("start") * 2.0
    } else {
        read("start") + read("end")
    }
}

// within a pixel of an end counts as the end
fn normalize(value: f64, max: f64) -> f64 {
    if max <= 0.0 {
        return 0.0;
    }
    let clamped = value.clamp(0.0, max);
    match (clamped <= 1.0, max - clamped <= 1.0) {
        (true, true) if clamped <= max - clamped => 0.0,
        (true, true) => max,
        (true, false) => 0.0,
        (false, true) => max,
        (false, false) => clamped,
    }
}

// an elastic overscroll squeezes the thumb against its end
fn overscroll(
    thumb: &HtmlElement,
    vertical: bool,
    from_start: f64,
    range: f64,
    content: f64,
    size: f64,
    travel: f64,
) -> f64 {
    let clamped = from_start.clamp(0.0, range);
    let over = from_start - clamped;
    let squeezed = (size * content / (content + over.abs())).max(MIN_THUMB_SIZE);
    let style = thumb.style();
    let name = if vertical {
        "--scroll-area-thumb-height"
    } else {
        "--scroll-area-thumb-width"
    };
    let _ = if over != 0.0 {
        style.set_property(name, &format!("{squeezed}px"))
    } else {
        style.remove_property(name).map(drop)
    };
    let at = if range > 0.0 {
        clamped / range * travel
    } else {
        0.0
    };
    at + if over > 0.0 { size - squeezed } else { 0.0 }
}

fn observe_resize(element: &web_sys::Element, measure: impl Fn() + 'static) {
    let callback = Closure::<dyn Fn()>::new(measure);
    let Ok(observer) = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()) else {
        return;
    };
    observer.observe(element);
    let held = StoredValue::new_local((observer, callback));
    on_cleanup(move || {
        held.try_with_value(|(observer, _)| observer.disconnect());
    });
}

fn state_attributes(
    context: ScrollAreaContext,
    scrolling: impl Fn() -> bool + Copy + Send + Sync + 'static,
) -> impl leptos::attr::Attribute {
    let flag = move |on: fn(&ScrollAreaContext) -> bool| move || on(&context).then_some("");
    view! {
        <{..}
            data-scrolling=move || scrolling().then_some("")
            data-has-overflow-x=flag(|context| !context.hidden.get()[0])
            data-has-overflow-y=flag(|context| !context.hidden.get()[1])
            data-overflow-x-start=flag(|context| context.edges.get()[0])
            data-overflow-x-end=flag(|context| context.edges.get()[1])
            data-overflow-y-start=flag(|context| context.edges.get()[2])
            data-overflow-y-end=flag(|context| context.edges.get()[3])
        />
    }
}

#[component(element = "div", skip(role))]
pub fn ScrollAreaRoot(
    #[prop(optional, into)] overflow_edge_threshold: Signal<f64>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = ScrollAreaContext {
        viewport: NodeRef::new(),
        scrollbars: [NodeRef::new(), NodeRef::new()],
        thumbs: [NodeRef::new(), NodeRef::new()],
        corner: NodeRef::new(),
        hovering: RwSignal::new(false),
        scrolling: [RwSignal::new(false), RwSignal::new(false)],
        measured: RwSignal::new(false),
        hidden: RwSignal::new([true, true]),
        edges: RwSignal::new([false; 4]),
        corner_size: RwSignal::new([0.0, 0.0]),
        thumb_size: RwSignal::new([0.0, 0.0]),
        threshold: Signal::derive(move || overflow_edge_threshold.get().max(0.0)),
        direction: use_direction(),
        drag: StoredValue::new(Drag::default()),
        position: StoredValue::new([0.0, 0.0]),
        timeouts: StoredValue::new([None, None]),
        touch: StoredValue::new(false),
        snap: StoredValue::new(None),
    };
    on_cleanup(move || {
        let timeouts = context.timeouts.try_get_value().unwrap_or_default();
        for timeout in timeouts.into_iter().flatten() {
            timeout.clear();
        }
    });
    let pointer = move |event: ev::PointerEvent, hovered: bool| {
        let touch = event.pointer_type() == "touch";
        context.touch.set_value(touch);
        if hovered && !touch {
            change(context.hovering, true);
        }
    };
    let corner = move |axis: usize| move || format!("{}px", context.corner_size.get()[axis]);
    view! {
        <div
            {..attrs}
            {..state_attributes(context, context.scrolling_any())}
            role="presentation"
            style:position="relative"
            style:--scroll-area-corner-width=corner(0)
            style:--scroll-area-corner-height=corner(1)
            on:pointerenter=move |event| pointer(event, true)
            on:pointermove=move |event| pointer(event, true)
            on:pointerdown=move |event| pointer(event, false)
            on:pointerleave=move |_| change(context.hovering, false)
        >
            <Provider value=context>{children.map(|children| children())}</Provider>
        </div>
    }
}

// the native scrollbars hide inline, base ui injects a stylesheet for it
#[component(element = "div", skip(role, tabindex))]
pub fn ScrollAreaViewport(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<ScrollAreaContext>();
    let viewport = context.viewport;
    // set by a wheel, a key or a pointer, so the scroll that follows is the user's
    let user = StoredValue::new(false);
    let settle = StoredValue::new(None::<TimeoutHandle>);
    let interacted = move || user.set_value(true);
    Effect::new(move || {
        context.hidden.track();
        context.direction.track();
        context.threshold.track();
        request_animation_frame(move || context.measure());
    });
    Effect::new(move || {
        let Some(element) = viewport.get() else {
            return;
        };
        if element.matches(":hover").unwrap_or(false) {
            change(context.hovering, true);
        }
        observe_resize(&element, move || context.measure());
    });
    on_cleanup(move || {
        if let Some(Some(timeout)) = settle.try_get_value() {
            timeout.clear();
        }
    });
    let scrolled = move |_| {
        let Some(element) = viewport.get_untracked() else {
            return;
        };
        context.measure();
        if context.touch.get_value() || user.get_value() {
            context.scrolled(&element);
        }
        let timeout = set_timeout_with_handle(
            move || {
                user.try_set_value(false);
            },
            Duration::from_millis(100),
        );
        if let Some(previous) = settle.get_value() {
            previous.clear();
        }
        settle.set_value(timeout.ok());
    };
    view! {
        <div
            {..attrs}
            {..state_attributes(context, context.scrolling_any())}
            node_ref=viewport
            role="presentation"
            tabindex=move || if context.hidden.get() == [true, true] { -1 } else { 0 }
            style:overflow="scroll"
            style:scrollbar-width="none"
            on:scroll=scrolled
            on:wheel=move |_| interacted()
            on:pointermove=move |_| interacted()
            on:pointerenter=move |_| interacted()
            on:keydown=move |_| interacted()
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div", skip(role))]
pub fn ScrollAreaContent(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<ScrollAreaContext>();
    let content = NodeRef::<html::Div>::new();
    Effect::new(move || {
        if let Some(element) = content.get() {
            observe_resize(&element, move || context.measure());
        }
    });
    view! {
        <div
            {..attrs}
            {..state_attributes(context, context.scrolling_any())}
            node_ref=content
            role="presentation"
            style:min-width="fit-content"
        >
            {children.map(|children| children())}
        </div>
    }
}

#[derive(Clone, Copy)]
struct ScrollbarContext {
    vertical: bool,
}

// hidden rather than unmounted while there's nothing to scroll, the axis is read once
#[component(element = "div")]
pub fn ScrollAreaScrollbar(
    #[prop(into, default = "vertical".into())] orientation: Signal<&'static str>,
    #[prop(optional, into)] keep_mounted: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<ScrollAreaContext>();
    let vertical = orientation.get_untracked() == "vertical";
    let axis = vertical as usize;
    let scrollbar = context.scrollbars[axis];
    // not delegated: a passive listener on the document couldn't cancel the page's scroll
    let wheel = on(
        ev::Custom::<ev::WheelEvent>::new("wheel"),
        move |event: ev::WheelEvent| {
            let Some(viewport) = context.viewport.get_untracked() else {
                return;
            };
            let delta = if vertical {
                event.delta_y()
            } else {
                event.delta_x()
            };
            if event.ctrl_key() || delta == 0.0 {
                return;
            }
            let extent = Extent::of(&viewport, vertical);
            let range = extent.range();
            let (min, max) = if !vertical && context.rtl() {
                (-range, 0.0)
            } else {
                (0.0, range)
            };
            if (extent.scroll <= min && delta < 0.0) || (extent.scroll >= max && delta > 0.0) {
                return;
            }
            event.prevent_default();
            scroll_to(&viewport, vertical, (extent.scroll + delta).clamp(min, max));
            context.scrolled(&viewport);
        },
    );
    let track = move |event: ev::PointerEvent| {
        if event.button() != 0 {
            return;
        }
        let (Some(viewport), Some(thumb), Some(bar)) = (
            context.viewport.get_untracked(),
            context.thumbs[axis].get_untracked(),
            scrollbar.get_untracked(),
        ) else {
            return;
        };
        let target = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::Node>().ok());
        if thumb.contains(target.as_ref()) {
            return;
        }
        let rect = bar.get_bounding_client_rect();
        let click = client(&event, vertical) - if vertical { rect.top() } else { rect.left() };
        let thumb_size = along(&thumb, vertical);
        let padding = offset(&bar, "padding", vertical);
        let margin = offset(&thumb, "margin", vertical);
        let travel = travel(&bar, &thumb, thumb_size, vertical);
        if travel <= 0.0 {
            return;
        }
        let ratio = (click - thumb_size / 2.0 - padding + margin / 2.0) / travel;
        let range = Extent::of(&viewport, vertical).range();
        context.disable_snap();
        let to = if !vertical && context.rtl() {
            -(1.0 - ratio) * range
        } else {
            ratio * range
        };
        scroll_to(&viewport, vertical, to);
        context.scrolled(&viewport);
        context.press(&event, vertical);
    };
    let size = move || format!("{}px", context.thumb_size.get()[axis]);
    view! {
        <div
            {..attrs}
            {..state_attributes(context, move || context.scrolling[axis].get())}
            {..wheel}
            node_ref=scrollbar
            aria-hidden="true"
            hidden=move || !keep_mounted.get() && context.hidden.get()[axis]
            data-orientation=orientation
            data-hovering=move || context.hovering.get().then_some("")
            style:position="absolute"
            style:touch-action="none"
            style:user-select="none"
            style:-webkit-user-select="none"
            style:visibility=move || {
                (!context.measured.get() && !keep_mounted.get()).then_some("hidden")
            }
            style:top=vertical.then_some("0")
            style:bottom=if vertical { "var(--scroll-area-corner-height)" } else { "0" }
            style:inset-inline-start=(!vertical).then_some("0")
            style:inset-inline-end=if vertical { "0" } else { "var(--scroll-area-corner-width)" }
            style:--scroll-area-thumb-height=move || vertical.then(size)
            style:--scroll-area-thumb-width=move || (!vertical).then(size)
            on:pointerdown=track
            on:mousedown=move |event: ev::MouseEvent| event.prevent_default()
            on:pointerup=move |event: ev::PointerEvent| context.release(&event)
            on:pointercancel=move |event: ev::PointerEvent| context.release(&event)
        >
            <Provider value=ScrollbarContext { vertical }>
                {children.map(|children| children())}
            </Provider>
        </div>
    }
}

#[component(element = "div")]
pub fn ScrollAreaThumb(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<ScrollAreaContext>();
    let vertical = expect_context::<ScrollbarContext>().vertical;
    let axis = vertical as usize;
    view! {
        <div
            {..attrs}
            node_ref=context.thumbs[axis]
            data-orientation=if vertical { "vertical" } else { "horizontal" }
            data-scrolling=move || context.scrolling[axis].get().then_some("")
            style:visibility=move || (!context.measured.get()).then_some("hidden")
            style:height=vertical.then_some("var(--scroll-area-thumb-height)")
            style:width=(!vertical).then_some("var(--scroll-area-thumb-width)")
            on:pointerdown=move |event: ev::PointerEvent| context.press(&event, vertical)
            on:pointermove=move |event: ev::PointerEvent| context.drag(&event)
            on:pointerup=move |event: ev::PointerEvent| context.release(&event)
            on:pointercancel=move |event: ev::PointerEvent| context.release(&event)
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn ScrollAreaCorner(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<ScrollAreaContext>();
    let size = move |axis: usize| move || format!("{}px", context.corner_size.get()[axis]);
    view! {
        <div
            {..attrs}
            node_ref=context.corner
            aria-hidden="true"
            hidden=move || context.hidden.get().contains(&true)
            style:position="absolute"
            style:bottom="0"
            style:inset-inline-end="0"
            style:width=size(0)
            style:height=size(1)
        >
            {children.map(|children| children())}
        </div>
    }
}
