use crate::native::{along, change, client, observe_resize};
use crate::{Direction, Orientation, keyword, use_direction};
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::tachys::html::event::on;
use leptos::wasm_bindgen::JsCast;
use leptos_kit_macros::component;
use std::ops::{Index, IndexMut};
use std::time::Duration;
use web_sys::HtmlElement;

const SCROLL_TIMEOUT: Duration = Duration::from_millis(500);
const MIN_THUMB_SIZE: f64 = 16.0;
const AXES: [Orientation; 2] = [Orientation::Horizontal, Orientation::Vertical];

#[derive(Clone, Copy, Default, PartialEq)]
struct PerAxis<T> {
    x: T,
    y: T,
}

impl<T> PerAxis<T> {
    fn from_fn(mut value: impl FnMut(Orientation) -> T) -> Self {
        Self {
            x: value(Orientation::Horizontal),
            y: value(Orientation::Vertical),
        }
    }
}

impl<T> Index<Orientation> for PerAxis<T> {
    type Output = T;

    fn index(&self, axis: Orientation) -> &T {
        match axis {
            Orientation::Horizontal => &self.x,
            Orientation::Vertical => &self.y,
        }
    }
}

impl<T> IndexMut<Orientation> for PerAxis<T> {
    fn index_mut(&mut self, axis: Orientation) -> &mut T {
        match axis {
            Orientation::Horizontal => &mut self.x,
            Orientation::Vertical => &mut self.y,
        }
    }
}

// whether the content overflows past either end of an axis
#[derive(Clone, Copy, Default, PartialEq)]
struct Edges {
    start: bool,
    end: bool,
}

// the viewport along one axis: what scrolls, what shows, how far it has gone
#[derive(Clone, Copy)]
struct Extent {
    content: f64,
    size: f64,
    scroll: f64,
}

impl Extent {
    fn of(viewport: &HtmlElement, axis: Orientation) -> Self {
        let [content, size, scroll] = match axis {
            Orientation::Horizontal => [
                viewport.scroll_width(),
                viewport.client_width(),
                viewport.scroll_left(),
            ],
            Orientation::Vertical => [
                viewport.scroll_height(),
                viewport.client_height(),
                viewport.scroll_top(),
            ],
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

#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "web-sys takes whole pixels"
)]
fn scroll_to(viewport: &HtmlElement, axis: Orientation, value: f64) {
    let value = value as i32;
    match axis {
        Orientation::Horizontal => viewport.set_scroll_left(value),
        Orientation::Vertical => viewport.set_scroll_top(value),
    }
}

// how far the thumb can travel in its scrollbar
fn travel(scrollbar: &HtmlElement, thumb: &HtmlElement, thumb_size: f64, axis: Orientation) -> f64 {
    along(scrollbar, axis)
        - thumb_size
        - offset(scrollbar, "padding", axis)
        - offset(thumb, "margin", axis)
}

#[derive(Clone, Copy, Default)]
struct Drag {
    pointer: Option<i32>,
    axis: Orientation,
    start: f64,
    scroll: f64,
}

#[derive(Clone, Copy)]
struct ScrollAreaContext {
    viewport: NodeRef<html::Div>,
    scrollbars: PerAxis<NodeRef<html::Div>>,
    thumbs: PerAxis<NodeRef<html::Div>>,
    corner: NodeRef<html::Div>,
    hovering: RwSignal<bool>,
    scrolling: PerAxis<RwSignal<bool>>,
    measured: RwSignal<bool>,
    hidden: RwSignal<PerAxis<bool>>,
    edges: RwSignal<PerAxis<Edges>>,
    corner_size: RwSignal<PerAxis<f64>>,
    thumb_size: RwSignal<PerAxis<f64>>,
    threshold: Signal<f64>,
    direction: Signal<Direction>,
    drag: StoredValue<Drag>,
    position: StoredValue<PerAxis<f64>>,
    timeouts: StoredValue<PerAxis<Option<TimeoutHandle>>>,
    touch: StoredValue<bool>,
    snap: StoredValue<Option<String>>,
}

impl ScrollAreaContext {
    fn rtl(self) -> bool {
        self.direction.get_untracked() == Direction::Rtl
    }

    fn scrolling_any(self) -> impl Fn() -> bool + Copy + Send + Sync + 'static {
        move || self.scrolling.x.get() || self.scrolling.y.get()
    }

    fn start_scrolling(self, axis: Orientation) {
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
        let now = PerAxis::from_fn(|axis| Extent::of(viewport, axis).scroll);
        let before = self.position.get_value();
        self.position.set_value(now);
        for axis in AXES {
            if now[axis] != before[axis] {
                self.start_scrolling(axis);
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

    fn press(self, event: &ev::PointerEvent, axis: Orientation) {
        if event.button() != 0 {
            return;
        }
        let drag = self.drag.get_value();
        let thumb = self.thumbs[drag.axis].get_untracked();
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
            axis,
            start: client(event, axis),
            scroll: Extent::of(&viewport, axis).scroll,
        });
        self.disable_snap();
        if let Some(thumb) = self.thumbs[axis].get_untracked() {
            let _ = thumb.set_pointer_capture(event.pointer_id());
        }
    }

    fn release(self, event: &ev::PointerEvent) {
        let drag = self.drag.get_value();
        if drag.pointer != Some(event.pointer_id()) {
            return;
        }
        self.drag.update_value(|drag| drag.pointer = None);
        change(self.scrolling[drag.axis], false);
        if let Some(snap) = self.snap.try_update_value(Option::take).flatten()
            && let Some(viewport) = self.viewport.get_untracked()
        {
            let _ = HtmlElement::style(&viewport).set_property("scroll-snap-type", &snap);
        }
        if let Some(thumb) = self.thumbs[drag.axis].get_untracked()
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
        let axis = drag.axis;
        let (Some(viewport), Some(thumb), Some(scrollbar)) = (
            self.viewport.get_untracked(),
            self.thumbs[axis].get_untracked(),
            self.scrollbars[axis].get_untracked(),
        ) else {
            return;
        };
        let travel = travel(&scrollbar, &thumb, along(&thumb, axis), axis);
        let moved = client(event, axis) - drag.start;
        let ratio = if travel <= 0.0 { 0.0 } else { moved / travel };
        let range = Extent::of(&viewport, axis).range();
        scroll_to(&viewport, axis, drag.scroll + ratio * range);
        event.prevent_default();
        self.start_scrolling(axis);
    }

    // base ui's computeThumbPosition: thumb sizes and places, the corner, which axes overflow
    fn measure(self) {
        let Some(Some(viewport)) = self.viewport.try_get_untracked() else {
            return;
        };
        change(self.measured, true);
        let rtl = self.rtl();
        let extents = PerAxis::from_fn(|axis| Extent::of(&viewport, axis));
        if extents.x.content == 0.0 || extents.y.content == 0.0 {
            return;
        }
        let hidden = PerAxis::from_fn(|axis| extents[axis].size >= extents[axis].content);
        let parts = PerAxis::from_fn(|axis| {
            let scrollbar = self.scrollbars[axis]
                .get_untracked()
                .filter(|bar| !bar.hidden())?;
            let thumb = self.thumbs[axis].get_untracked().map(HtmlElement::from);
            Some((HtmlElement::from(scrollbar), thumb))
        });
        // each scrollbar's thickness, where the other one meets it
        let corner = if hidden.x || hidden.y {
            PerAxis::default()
        } else {
            PerAxis {
                x: parts
                    .y
                    .as_ref()
                    .map_or(0.0, |(bar, _)| along(bar, Orientation::Horizontal)),
                y: parts
                    .x
                    .as_ref()
                    .map_or(0.0, |(bar, _)| along(bar, Orientation::Vertical)),
            }
        };
        // until the corner has a size, the scrollbars still reach into it
        let unmeasured = self.corner_size.get_untracked() == PerAxis::default();
        let threshold = self.threshold.get_untracked();
        let mut edges = PerAxis::<Edges>::default();
        let mut sizes = PerAxis::<f64>::default();
        for axis in AXES {
            let extent = extents[axis];
            let range = extent.range();
            let from_start = if rtl && axis == Orientation::Horizontal {
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
            edges[axis] = Edges {
                start: !hidden[axis] && start > threshold,
                end: !hidden[axis] && end > threshold,
            };

            let (room, places) = match &parts[axis] {
                Some((bar, thumb)) => {
                    let inner = offset(bar, "padding", axis)
                        + thumb
                            .as_ref()
                            .map_or(0.0, |thumb| offset(thumb, "margin", axis));
                    let ideal = if hidden[axis] { 0.0 } else { extent.size } - inner;
                    let reach = along(bar, axis) - if unmeasured { corner[axis] } else { 0.0 };
                    (ideal.min(reach), thumb.as_ref().map(|thumb| (bar, thumb)))
                }
                None => (if hidden[axis] { 0.0 } else { extent.size }, None),
            };
            let size = (room * extent.size / extent.content).max(MIN_THUMB_SIZE);
            sizes[axis] = size;
            if let Some((bar, thumb)) = places {
                let travel = travel(bar, thumb, size, axis);
                let at = overscroll(thumb, axis, from_start, range, extent.content, size, travel);
                let transform = match axis {
                    Orientation::Horizontal => {
                        format!("translate3d({}px,0,0)", if rtl { -at } else { at })
                    }
                    Orientation::Vertical => format!("translate3d(0,{at}px,0)"),
                };
                let _ = thumb.style().set_property("transform", &transform);
            }
            let style = HtmlElement::style(&viewport);
            let name = match axis {
                Orientation::Horizontal => "x",
                Orientation::Vertical => "y",
            };
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

// padding or margin along an axis, and base ui counts a horizontal thumb's start margin twice
fn offset(element: &HtmlElement, property: &str, axis: Orientation) -> f64 {
    let Some(style) = window().get_computed_style(element).ok().flatten() else {
        return 0.0;
    };
    let side = match axis {
        Orientation::Horizontal => "inline",
        Orientation::Vertical => "block",
    };
    let read = |edge: &str| {
        style
            .get_property_value(&format!("{property}-{side}-{edge}"))
            .ok()
            .and_then(|value| value.trim_end_matches("px").parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    if axis == Orientation::Horizontal && property == "margin" {
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
        (_, true) => max,
        (true, false) => 0.0,
        (false, false) => clamped,
    }
}

// an elastic overscroll squeezes the thumb against its end
fn overscroll(
    thumb: &HtmlElement,
    axis: Orientation,
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
    let name = match axis {
        Orientation::Horizontal => "--scroll-area-thumb-width",
        Orientation::Vertical => "--scroll-area-thumb-height",
    };
    let _ = if over == 0.0 {
        style.remove_property(name).map(drop)
    } else {
        style.set_property(name, &format!("{squeezed}px"))
    };
    let at = if range > 0.0 {
        clamped / range * travel
    } else {
        0.0
    };
    at + if over > 0.0 { size - squeezed } else { 0.0 }
}

fn state_attributes(
    context: ScrollAreaContext,
    scrolling: impl Fn() -> bool + Copy + Send + Sync + 'static,
) -> impl leptos::attr::Attribute {
    let flag = move |on: fn(&ScrollAreaContext) -> bool| move || on(&context).then_some("");
    view! {
        <{..}
            data-scrolling=move || scrolling().then_some("")
            data-has-overflow-x=flag(|context| !context.hidden.get().x)
            data-has-overflow-y=flag(|context| !context.hidden.get().y)
            data-overflow-x-start=flag(|context| context.edges.get().x.start)
            data-overflow-x-end=flag(|context| context.edges.get().x.end)
            data-overflow-y-start=flag(|context| context.edges.get().y.start)
            data-overflow-y-end=flag(|context| context.edges.get().y.end)
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
        scrollbars: PerAxis::from_fn(|_| NodeRef::new()),
        thumbs: PerAxis::from_fn(|_| NodeRef::new()),
        corner: NodeRef::new(),
        hovering: RwSignal::new(false),
        scrolling: PerAxis::from_fn(|_| RwSignal::new(false)),
        measured: RwSignal::new(false),
        hidden: RwSignal::new(PerAxis { x: true, y: true }),
        edges: RwSignal::new(PerAxis::default()),
        corner_size: RwSignal::new(PerAxis::default()),
        thumb_size: RwSignal::new(PerAxis::default()),
        threshold: Signal::derive(move || overflow_edge_threshold.get().max(0.0)),
        direction: use_direction(),
        drag: StoredValue::new(Drag::default()),
        position: StoredValue::new(PerAxis::default()),
        timeouts: StoredValue::new(PerAxis::default()),
        touch: StoredValue::new(false),
        snap: StoredValue::new(None),
    };
    on_cleanup(move || {
        let timeouts = context.timeouts.try_get_value().unwrap_or_default();
        for timeout in [timeouts.x, timeouts.y].into_iter().flatten() {
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
    let corner = move |axis: Orientation| move || format!("{}px", context.corner_size.get()[axis]);
    view! {
        <div
            {..attrs}
            {..state_attributes(context, context.scrolling_any())}
            role="presentation"
            style:position="relative"
            style:--scroll-area-corner-width=corner(Orientation::Horizontal)
            style:--scroll-area-corner-height=corner(Orientation::Vertical)
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
    let tabindex = move || {
        let hidden = context.hidden.get();
        if hidden.x && hidden.y { -1 } else { 0 }
    };
    view! {
        <div
            {..attrs}
            {..state_attributes(context, context.scrolling_any())}
            node_ref=viewport
            role="presentation"
            tabindex=tabindex
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
    let node = NodeRef::<html::Div>::new();
    Effect::new(move || {
        if let Some(element) = node.get() {
            observe_resize(&element, move || context.measure());
        }
    });
    view! {
        <div
            {..attrs}
            {..state_attributes(context, context.scrolling_any())}
            node_ref=node
            role="presentation"
            style:min-width="fit-content"
        >
            {children.map(|children| children())}
        </div>
    }
}

#[derive(Clone, Copy)]
struct ScrollbarContext {
    axis: Orientation,
}

// hidden rather than unmounted while there's nothing to scroll, the axis is read once
#[component(element = "div")]
pub fn ScrollAreaScrollbar(
    #[prop(into, default = Orientation::Vertical.into())] orientation: Signal<Orientation>,
    #[prop(optional, into)] keep_mounted: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<ScrollAreaContext>();
    let axis = orientation.get_untracked();
    let vertical = axis == Orientation::Vertical;
    let scrollbar = context.scrollbars[axis];
    // not delegated: a passive listener on the document couldn't cancel the page's scroll
    let wheel = on(
        ev::Custom::<ev::WheelEvent>::new("wheel"),
        move |event: ev::WheelEvent| {
            let Some(viewport) = context.viewport.get_untracked() else {
                return;
            };
            let delta = match axis {
                Orientation::Horizontal => event.delta_x(),
                Orientation::Vertical => event.delta_y(),
            };
            if event.ctrl_key() || delta == 0.0 {
                return;
            }
            let extent = Extent::of(&viewport, axis);
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
            scroll_to(&viewport, axis, (extent.scroll + delta).clamp(min, max));
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
        let click = client(&event, axis) - if vertical { rect.top() } else { rect.left() };
        let thumb_size = along(&thumb, axis);
        let padding = offset(&bar, "padding", axis);
        let margin = offset(&thumb, "margin", axis);
        let travel = travel(&bar, &thumb, thumb_size, axis);
        if travel <= 0.0 {
            return;
        }
        let ratio = (click - thumb_size / 2.0 - padding + margin / 2.0) / travel;
        let range = Extent::of(&viewport, axis).range();
        context.disable_snap();
        let to = if !vertical && context.rtl() {
            -(1.0 - ratio) * range
        } else {
            ratio * range
        };
        scroll_to(&viewport, axis, to);
        context.scrolled(&viewport);
        context.press(&event, axis);
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
            data-orientation=keyword(orientation)
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
            <Provider value=ScrollbarContext { axis }>
                {children.map(|children| children())}
            </Provider>
        </div>
    }
}

#[component(element = "div")]
pub fn ScrollAreaThumb(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<ScrollAreaContext>();
    let axis = expect_context::<ScrollbarContext>().axis;
    let vertical = axis == Orientation::Vertical;
    view! {
        <div
            {..attrs}
            node_ref=context.thumbs[axis]
            data-orientation=axis.as_str()
            data-scrolling=move || context.scrolling[axis].get().then_some("")
            style:visibility=move || (!context.measured.get()).then_some("hidden")
            style:height=vertical.then_some("var(--scroll-area-thumb-height)")
            style:width=(!vertical).then_some("var(--scroll-area-thumb-width)")
            on:pointerdown=move |event: ev::PointerEvent| context.press(&event, axis)
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
    let size = move |axis: Orientation| move || format!("{}px", context.corner_size.get()[axis]);
    view! {
        <div
            {..attrs}
            node_ref=context.corner
            aria-hidden="true"
            hidden=move || {
                let hidden = context.hidden.get();
                hidden.x || hidden.y
            }
            style:position="absolute"
            style:bottom="0"
            style:inset-inline-end="0"
            style:width=size(Orientation::Horizontal)
            style:height=size(Orientation::Vertical)
        >
            {children.map(|children| children())}
        </div>
    }
}
