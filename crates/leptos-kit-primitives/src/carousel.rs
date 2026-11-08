use crate::native::change;
use crate::{Align, Orientation, keyword};
use leptos::attr::Attribute;
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::tachys::html::node_ref::node_ref;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;
use leptos_kit_macros::component;
use std::sync::Arc;
use std::time::Duration;
use web_sys::{HtmlElement, ScrollBehavior, ScrollToOptions};

// embla takes a snap this close to an edge for the edge
const TOLERANCE: f64 = 2.0;
// embla's window for telling a flick from a drag, in milliseconds
const FLICK_WINDOW: f64 = 170.0;
// embla's flick force from which a drag moves on by one snap
const NEXT_SNAP_FORCE: f64 = 20.0;
// embla's boost of a mouse flick, snapping and free
const SNAP_BOOST: f64 = 300.0;
const FREE_BOOST: f64 = 500.0;
// without scrollend, how long a scroll has to rest to count as settled
const SETTLE_DELAY: Duration = Duration::from_millis(150);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ContainScroll {
    #[default]
    TrimSnaps,
    KeepSnaps,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SlidesToScroll {
    Count(usize),
    Auto,
}

impl Default for SlidesToScroll {
    fn default() -> Self {
        Self::Count(1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "embla's options, one flag each"
)]
pub struct CarouselOptions {
    pub align: Align,
    pub r#loop: bool,
    pub drag_free: bool,
    pub skip_snaps: bool,
    pub contain_scroll: ContainScroll,
    pub slides_to_scroll: SlidesToScroll,
    pub start_index: usize,
    pub watch_drag: bool,
    pub drag_threshold: f64,
}

impl Default for CarouselOptions {
    fn default() -> Self {
        Self {
            align: Align::Center,
            r#loop: false,
            drag_free: false,
            skip_snaps: false,
            contain_scroll: ContainScroll::TrimSnaps,
            slides_to_scroll: SlidesToScroll::Count(1),
            start_index: 0,
            watch_drag: true,
            drag_threshold: 10.0,
        }
    }
}

pub trait CarouselPlugin: Send + Sync + 'static {
    fn init(&self, api: CarouselApi);
    fn destroy(&self) {}
}

#[derive(Clone)]
pub struct AnyCarouselPlugin(Arc<dyn CarouselPlugin>);

impl<P: CarouselPlugin> From<P> for AnyCarouselPlugin {
    fn from(plugin: P) -> Self {
        Self(Arc::new(plugin))
    }
}

impl std::fmt::Debug for AnyCarouselPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyCarouselPlugin").finish_non_exhaustive()
    }
}

#[derive(Clone, Default)]
struct Layout {
    axis: Orientation,
    rtl: bool,
    view: f64,
    max: f64,
    // per slide in dom order: its size, where it starts as laid out, and where it would start unrotated
    sizes: Vec<f64>,
    starts: Vec<f64>,
    logical: Vec<f64>,
    gap: f64,
    cycle: f64,
    // per snap: where it sits, the first slide of its group and the group's size
    snaps: Vec<f64>,
    firsts: Vec<usize>,
    spans: Vec<f64>,
}

#[derive(Clone, Copy)]
struct Drag {
    pointer: i32,
    from: f64,
    position: f64,
    index: usize,
    sample: (f64, f64),
    last: (f64, f64),
    moved: bool,
    captured: bool,
}

#[derive(Clone, Copy)]
struct Carousel {
    orientation: Signal<Orientation>,
    options: Signal<CarouselOptions>,
    scroller: NodeRef<html::Div>,
    list: RwSignal<Vec<f64>>,
    selected: RwSignal<usize>,
    previous: RwSignal<usize>,
    progress: RwSignal<f64>,
    looping: RwSignal<bool>,
    layout: StoredValue<Layout>,
    // the snap and position a scroll we started is heading for, so a scrollend on the way doesn't settle it
    heading: StoredValue<Option<(usize, f64)>>,
    drag: StoredValue<Option<Drag>>,
    // a mouse drag turns snapping off until its scroll settles
    held: StoredValue<bool>,
    prevent_click: StoredValue<bool>,
    // the slide in the first slot while looping
    rotation: StoredValue<usize>,
    scrollend: StoredValue<bool>,
    settling: StoredValue<Option<TimeoutHandle>>,
}

#[derive(Clone, Copy)]
pub struct CarouselApi(StoredValue<Carousel>);

impl std::fmt::Debug for CarouselApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CarouselApi").finish_non_exhaustive()
    }
}

pub fn use_carousel() -> CarouselApi {
    expect_context::<CarouselApi>()
}

impl CarouselApi {
    fn carousel(self) -> Option<Carousel> {
        self.0.try_get_value()
    }

    pub fn orientation(self) -> Orientation {
        self.carousel()
            .map_or_else(Orientation::default, |carousel| carousel.orientation.get())
    }

    pub fn scroll_prev(self) {
        self.step(false, ScrollBehavior::Smooth);
    }

    pub fn scroll_next(self) {
        self.step(true, ScrollBehavior::Smooth);
    }

    pub fn scroll_to(self, index: usize) {
        self.go(index, ScrollBehavior::Smooth);
    }

    pub fn can_scroll_prev(self) -> bool {
        self.carousel().is_some_and(|carousel| {
            carousel.list.with(Vec::len) > 1
                && (carousel.looping.get() || carousel.selected.get() > 0)
        })
    }

    pub fn can_scroll_next(self) -> bool {
        self.carousel().is_some_and(|carousel| {
            let count = carousel.list.with(Vec::len);
            count > 1 && (carousel.looping.get() || carousel.selected.get() + 1 < count)
        })
    }

    pub fn selected_scroll_snap(self) -> usize {
        self.carousel()
            .map_or(0, |carousel| carousel.selected.get())
    }

    pub fn previous_scroll_snap(self) -> usize {
        self.carousel()
            .map_or(0, |carousel| carousel.previous.get())
    }

    pub fn scroll_snap_list(self) -> Vec<f64> {
        self.carousel()
            .map(|carousel| carousel.list.get())
            .unwrap_or_default()
    }

    pub fn scroll_progress(self) -> f64 {
        self.carousel()
            .map_or(0.0, |carousel| carousel.progress.get())
    }

    pub fn container_node(self) -> Option<HtmlElement> {
        self.carousel()?.scroller.get_untracked().map(Into::into)
    }

    pub(crate) fn watches_drag(self) -> bool {
        self.carousel().is_some_and(|carousel| {
            carousel
                .options
                .with_untracked(|options| options.watch_drag)
        })
    }

    pub(crate) fn step(self, forward: bool, behavior: ScrollBehavior) {
        let Some(carousel) = self.carousel() else {
            return;
        };
        if let Some(index) = self.neighbour(carousel.selected.get_untracked(), forward) {
            self.go(index, behavior);
        }
    }

    pub(crate) fn go(self, index: usize, behavior: ScrollBehavior) {
        let (Some(carousel), Some(element)) = (self.carousel(), self.container_node()) else {
            return;
        };
        if index >= carousel.list.with_untracked(Vec::len) {
            return;
        }
        if carousel.looping.get_untracked() && !self.reachable(index) {
            self.recenter(carousel.selected.get_untracked(), &element);
        }
        let Some(position) = carousel.layout.with_value(|layout| {
            layout
                .snaps
                .get(index)
                .map(|snap| snap.clamp(0.0, layout.max))
        }) else {
            return;
        };
        self.select(index);
        self.head(index, position, behavior, &element);
    }

    fn neighbour(self, index: usize, forward: bool) -> Option<usize> {
        let carousel = self.carousel()?;
        let count = carousel.list.with_untracked(Vec::len);
        let looping = carousel.looping.get_untracked();
        if count < 2 {
            return None;
        }
        match (forward, index) {
            (true, index) if index + 1 < count => Some(index + 1),
            (true, _) => looping.then_some(0),
            (false, 0) => looping.then_some(count - 1),
            (false, index) => Some(index - 1),
        }
    }

    fn reachable(self, index: usize) -> bool {
        self.carousel().is_some_and(|carousel| {
            carousel.layout.with_value(|layout| {
                layout
                    .snaps
                    .get(index)
                    .is_some_and(|snap| *snap >= -TOLERANCE && *snap <= layout.max + TOLERANCE)
            })
        })
    }

    fn nearest(self, position: f64) -> Option<usize> {
        self.carousel()?.layout.with_value(|layout| {
            layout
                .snaps
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| (*a - position).abs().total_cmp(&(*b - position).abs()))
                .map(|(index, _)| index)
        })
    }

    fn select(self, index: usize) {
        let Some(carousel) = self.carousel() else {
            return;
        };
        let current = carousel.selected.get_untracked();
        if current != index {
            carousel.previous.set(current);
            carousel.selected.set(index);
        }
    }

    fn position(self, element: &HtmlElement) -> f64 {
        let Some(carousel) = self.carousel() else {
            return 0.0;
        };
        let (axis, rtl) = carousel
            .layout
            .with_value(|layout| (layout.axis, layout.rtl));
        let name = match axis {
            Orientation::Horizontal => "scrollLeft",
            Orientation::Vertical => "scrollTop",
        };
        let scrolled = web_sys::js_sys::Reflect::get(element, &name.into())
            .ok()
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0);
        // a right-to-left scroller counts down from 0
        if rtl { -scrolled } else { scrolled }
    }

    fn scroll(self, element: &HtmlElement, position: f64, behavior: ScrollBehavior) {
        let Some(carousel) = self.carousel() else {
            return;
        };
        let (axis, rtl) = carousel
            .layout
            .with_value(|layout| (layout.axis, layout.rtl));
        let options = ScrollToOptions::new();
        options.set_behavior(behavior);
        // the other axis back at 0, it's left over after the orientation changes
        let (left, top) = match axis {
            Orientation::Horizontal => (if rtl { -position } else { position }, 0.0),
            Orientation::Vertical => (0.0, position),
        };
        options.set_left(left);
        options.set_top(top);
        element.scroll_to_with_scroll_to_options(&options);
    }

    fn head(self, index: usize, position: f64, behavior: ScrollBehavior, element: &HtmlElement) {
        let Some(carousel) = self.carousel() else {
            return;
        };
        carousel.heading.set_value(Some((index, position)));
        let near = (self.position(element) - position).abs() < 1.0;
        if !near {
            self.scroll(element, position, behavior);
        }
        if near || behavior == ScrollBehavior::Instant {
            self.settled();
        }
    }

    fn snap_type(self) -> &'static str {
        let Some(carousel) = self.carousel() else {
            return "none";
        };
        if carousel.options.with_untracked(|options| options.drag_free) {
            return "none";
        }
        match carousel.orientation.get_untracked() {
            Orientation::Horizontal => "x mandatory",
            Orientation::Vertical => "y mandatory",
        }
    }

    fn hold_snap(self, element: &HtmlElement) {
        let Some(carousel) = self.carousel() else {
            return;
        };
        carousel.held.set_value(true);
        let style = element.style();
        let _ = style.set_property("scroll-snap-type", "none");
        let _ = style.set_property("user-select", "none");
        let _ = style.set_property("-webkit-user-select", "none");
    }

    fn release_snap(self, element: &HtmlElement) {
        let Some(carousel) = self.carousel() else {
            return;
        };
        if !carousel.held.get_value() {
            return;
        }
        carousel.held.set_value(false);
        let style = element.style();
        let _ = style.set_property("scroll-snap-type", self.snap_type());
        let _ = style.remove_property("user-select");
        let _ = style.remove_property("-webkit-user-select");
    }

    fn measure(self) {
        let (Some(carousel), Some(element)) = (self.carousel(), self.container_node()) else {
            return;
        };
        let options = carousel.options.get_untracked();
        let axis = carousel.orientation.get_untracked();
        let rtl = axis == Orientation::Horizontal
            && window()
                .get_computed_style(&element)
                .ok()
                .flatten()
                .is_some_and(|style| {
                    style
                        .get_property_value("direction")
                        .is_ok_and(|d| d == "rtl")
                });
        let slides = slides(&element);
        let count = slides.len();
        let position = |element: &HtmlElement| {
            let name = match axis {
                Orientation::Horizontal => "scrollLeft",
                Orientation::Vertical => "scrollTop",
            };
            let scrolled = web_sys::js_sys::Reflect::get(element, &name.into())
                .ok()
                .and_then(|value| value.as_f64())
                .unwrap_or(0.0);
            if rtl { -scrolled } else { scrolled }
        };
        let read = |slides: &[HtmlElement]| {
            let rect = element.get_bounding_client_rect();
            let scrolled = position(&element);
            let (view, origin) = match (axis, rtl) {
                (Orientation::Horizontal, false) => (rect.width(), rect.left()),
                (Orientation::Horizontal, true) => (rect.width(), rect.right()),
                (Orientation::Vertical, _) => (rect.height(), rect.top()),
            };
            let (starts, sizes): (Vec<f64>, Vec<f64>) = slides
                .iter()
                .map(|slide| {
                    let rect = slide.get_bounding_client_rect();
                    let (start, size) = match (axis, rtl) {
                        (Orientation::Horizontal, false) => (rect.left() - origin, rect.width()),
                        (Orientation::Horizontal, true) => (origin - rect.right(), rect.width()),
                        (Orientation::Vertical, _) => (rect.top() - origin, rect.height()),
                    };
                    (start + scrolled, size)
                })
                .unzip();
            (view, starts, sizes)
        };
        let (view, mut starts, sizes) = read(&slides);
        let max = f64::from(match axis {
            Orientation::Horizontal => element.scroll_width() - element.client_width(),
            Orientation::Vertical => element.scroll_height() - element.client_height(),
        })
        .max(0.0);

        let rotation = carousel.rotation.get_value() % count.max(1);
        let gap = if count > 1 {
            let (first, second) = (rotation, (rotation + 1) % count);
            (starts[second] - starts[first] - sizes[first]).max(0.0)
        } else {
            0.0
        };
        // where each slide would start in dom order, whatever order they're laid out in
        let unrotated: Vec<f64> = sizes
            .iter()
            .scan(0.0, |at, size| {
                let start = *at;
                *at += size + gap;
                Some(start)
            })
            .collect();
        let spans = |starts: &[f64], groups: &[(usize, usize)]| -> Vec<f64> {
            groups
                .iter()
                .map(|&(first, end)| starts[end - 1] + sizes[end - 1] - starts[first])
                .collect()
        };
        let extent = unrotated
            .last()
            .zip(sizes.last())
            .map_or(0.0, |(start, size)| start + size);
        let looping = options.r#loop && count > 1 && {
            let groups = groups(options.slides_to_scroll, &unrotated, &sizes, view);
            let widest = spans(&unrotated, &groups).into_iter().fold(0.0, f64::max);
            extent - view + TOLERANCE >= 2.0 * widest
        };

        let rotation = if looping { rotation } else { 0 };
        carousel.rotation.set_value(rotation);
        let mut reordered = false;
        for (index, slide) in slides.iter().enumerate() {
            let order = looping.then(|| ((index + count - rotation) % count).to_string());
            reordered |= put(slide, "order", order.as_deref());
        }
        if reordered {
            starts = read(&slides).1;
        }
        let logical = if looping { unrotated } else { starts.clone() };
        let groups = groups(options.slides_to_scroll, &logical, &sizes, view);
        let spans = spans(&logical, &groups);

        let (margin, other) = match axis {
            Orientation::Horizontal => ("scroll-margin-inline-start", "scroll-margin-block-start"),
            Orientation::Vertical => ("scroll-margin-block-start", "scroll-margin-inline-start"),
        };
        let stop = if options.skip_snaps {
            "normal"
        } else {
            "always"
        };
        for (&(first, end), span) in groups.iter().zip(&spans) {
            for (index, slide) in slides.iter().enumerate().take(end).skip(first) {
                let (align, moved) = if index != first {
                    ("none", None)
                } else if end - first == 1 {
                    (options.align.as_str(), None)
                } else {
                    // a group snaps by its first slide, moved by the group's alignment
                    (
                        "start",
                        Some(format!("{}px", offset(options.align, view, *span))),
                    )
                };
                put(slide, other, None);
                put(slide, "scroll-snap-stop", Some(stop));
                put(slide, "scroll-snap-align", Some(align));
                put(slide, margin, moved.as_deref());
            }
        }

        let aligned: Vec<f64> = groups
            .iter()
            .zip(&spans)
            .map(|(&(first, _), span)| starts[first] - offset(options.align, view, *span))
            .collect();
        let firsts: Vec<usize> = groups.iter().map(|&(first, _)| first).collect();
        let (snaps, kept) = if looping || aligned.is_empty() {
            (aligned, 0..groups.len())
        } else {
            contain(&aligned, max, options.contain_scroll)
        };
        let firsts = firsts[kept.clone()].to_vec();
        let spans = spans[kept].to_vec();
        let cycle = extent + gap;
        let list: Vec<f64> = if looping {
            firsts.iter().map(|first| logical[*first] / cycle).collect()
        } else {
            snaps
                .iter()
                .map(|snap| if max > 0.0 { snap / max } else { 0.0 })
                .collect()
        };
        let snap_count = snaps.len();

        carousel.layout.set_value(Layout {
            axis,
            rtl,
            view,
            max,
            sizes,
            starts,
            logical,
            gap,
            cycle,
            snaps,
            firsts,
            spans,
        });
        change(carousel.list, list);
        change(carousel.looping, looping);
        if snap_count > 0 && carousel.selected.get_untracked() >= snap_count {
            self.select(snap_count - 1);
        }
        self.track(position(&element));
    }

    // puts the slides round the snap so there's room to scroll either way, keeping what shows in place
    fn recenter(self, index: usize, element: &HtmlElement) -> f64 {
        let Some(carousel) = self.carousel() else {
            return 0.0;
        };
        if !carousel.looping.get_untracked() {
            return 0.0;
        }
        let Some((first, rotation, before)) = carousel.layout.with_value(|layout| {
            let count = layout.sizes.len();
            let first = *layout.firsts.get(index)?;
            let span = *layout.spans.get(index)?;
            let align = carousel.options.with_untracked(|options| options.align);
            let lead = offset(align, layout.view, span);
            let distance = |rotation: usize| {
                let start: f64 = (0..count)
                    .map(|slot| (rotation + slot) % count)
                    .take_while(|slide| *slide != first)
                    .map(|slide| layout.sizes[slide] + layout.gap)
                    .sum();
                (start - lead - layout.max / 2.0).abs()
            };
            let rotation = (0..count).min_by(|a, b| distance(*a).total_cmp(&distance(*b)))?;
            Some((first, rotation, *layout.starts.get(first)?))
        }) else {
            return 0.0;
        };
        if rotation == carousel.rotation.get_value() {
            return 0.0;
        }
        let position = self.position(element);
        carousel.rotation.set_value(rotation);
        self.measure();
        let after = carousel
            .layout
            .with_value(|layout| layout.starts.get(first).copied())
            .unwrap_or(before);
        let shift = after - before;
        self.scroll(element, position + shift, ScrollBehavior::Instant);
        self.track(position + shift);
        shift
    }

    fn track(self, position: f64) {
        let Some(carousel) = self.carousel() else {
            return;
        };
        let looping = carousel.looping.get_untracked();
        let nearest = self.nearest(position);
        let progress = carousel.layout.with_value(|layout| {
            if looping {
                nearest.map_or(0.0, |index| {
                    let start = layout
                        .firsts
                        .get(index)
                        .map_or(0.0, |first| layout.logical[*first]);
                    let moved = position - layout.snaps[index];
                    ((start + moved) / layout.cycle).rem_euclid(1.0)
                })
            } else if layout.max > 0.0 {
                (position / layout.max).clamp(0.0, 1.0)
            } else {
                0.0
            }
        });
        change(carousel.progress, progress);
    }

    fn init(self, element: &HtmlElement) {
        self.measure();
        let Some(carousel) = self.carousel() else {
            return;
        };
        let count = carousel.list.with_untracked(Vec::len);
        if count == 0 {
            return;
        }
        let start = carousel
            .options
            .with_untracked(|options| options.start_index)
            .min(count - 1);
        carousel.selected.set(start);
        carousel.previous.set(start);
        self.recenter(start, element);
        if let Some(snap) = carousel
            .layout
            .with_value(|layout| layout.snaps.get(start).copied())
        {
            self.scroll(element, snap, ScrollBehavior::Instant);
        }
        self.track(self.position(element));
    }

    // after a resize or a change of slides, back onto the snap it was on
    fn remeasure(self) {
        self.measure();
        let (Some(carousel), Some(element)) = (self.carousel(), self.container_node()) else {
            return;
        };
        if carousel.drag.get_value().is_some() || carousel.heading.get_value().is_some() {
            return;
        }
        let selected = carousel.selected.get_untracked();
        self.recenter(selected, &element);
        let snap = carousel.layout.with_value(|layout| {
            layout
                .snaps
                .get(selected)
                .map(|snap| snap.clamp(0.0, layout.max))
        });
        if let Some(snap) = snap
            && (self.position(&element) - snap).abs() > 1.0
        {
            self.scroll(&element, snap, ScrollBehavior::Instant);
        }
    }

    fn scrolled(self) {
        let (Some(carousel), Some(element)) = (self.carousel(), self.container_node()) else {
            return;
        };
        let position = self.position(&element);
        self.track(position);
        if carousel.heading.get_value().is_none()
            && carousel.drag.get_value().is_none()
            && let Some(nearest) = self.nearest(position)
        {
            self.select(nearest);
        }
        if !carousel.scrollend.get_value() {
            if let Some(Some(handle)) = carousel.settling.try_update_value(Option::take) {
                handle.clear();
            }
            let handle = set_timeout_with_handle(move || self.settled(), SETTLE_DELAY).ok();
            carousel.settling.set_value(handle);
        }
    }

    fn settled(self) {
        let (Some(carousel), Some(element)) = (self.carousel(), self.container_node()) else {
            return;
        };
        if carousel.drag.get_value().is_some() {
            return;
        }
        let position = self.position(&element);
        if let Some((_, heading)) = carousel.heading.get_value()
            && (position - heading).abs() > TOLERANCE
        {
            return;
        }
        carousel.heading.set_value(None);
        self.release_snap(&element);
        if let Some(nearest) = self.nearest(position) {
            self.select(nearest);
        }
        self.recenter(carousel.selected.get_untracked(), &element);
    }

    fn interrupted(self) {
        if let Some(carousel) = self.carousel() {
            carousel.heading.set_value(None);
        }
    }

    fn point(self, event: &ev::PointerEvent) -> f64 {
        f64::from(
            match self
                .carousel()
                .map(|carousel| carousel.orientation.get_untracked())
            {
                Some(Orientation::Vertical) => event.client_y(),
                _ => event.client_x(),
            },
        )
    }

    fn pointer_down(self, event: &ev::PointerEvent) {
        let Some(carousel) = self.carousel() else {
            return;
        };
        carousel.heading.set_value(None);
        carousel.prevent_click.set_value(false);
        let watch_drag = carousel
            .options
            .with_untracked(|options| options.watch_drag);
        if !watch_drag || event.pointer_type() != "mouse" || event.button() != 0 {
            return;
        }
        // embla leaves the pointer to fields
        let on_field = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            .is_some_and(|target| {
                matches!(target.tag_name().as_str(), "INPUT" | "SELECT" | "TEXTAREA")
            });
        let Some(element) = self.container_node() else {
            return;
        };
        if on_field {
            return;
        }
        let point = self.point(event);
        let time = event.time_stamp();
        carousel.drag.set_value(Some(Drag {
            pointer: event.pointer_id(),
            from: point,
            position: self.position(&element),
            index: carousel.selected.get_untracked(),
            sample: (point, time),
            last: (point, time),
            moved: false,
            captured: false,
        }));
    }

    fn pointer_move(self, event: &ev::PointerEvent) {
        let (Some(carousel), Some(element)) = (self.carousel(), self.container_node()) else {
            return;
        };
        let Some(mut drag) = carousel.drag.get_value() else {
            return;
        };
        if event.pointer_id() != drag.pointer {
            return;
        }
        let point = self.point(event);
        let time = event.time_stamp();
        let moved = point - drag.from;
        if !drag.moved {
            if moved == 0.0 {
                return;
            }
            drag.moved = true;
            self.hold_snap(&element);
        }
        let threshold = carousel
            .options
            .with_untracked(|options| options.drag_threshold);
        if !drag.captured && moved.abs() > threshold {
            // only past the threshold, a capture earlier would take the click from what was pressed
            drag.captured = element.set_pointer_capture(drag.pointer).is_ok();
            carousel.prevent_click.set_value(true);
        }
        let (max, rtl, edge) = carousel.layout.with_value(|layout| {
            let widest = layout.sizes.iter().copied().fold(0.0, f64::max);
            (layout.max, layout.rtl, widest)
        });
        let mut position = if rtl {
            drag.position + moved
        } else {
            drag.position - moved
        }
        .clamp(0.0, max);
        if carousel.looping.get_untracked()
            && (position < edge || position > max - edge)
            && let Some(nearest) = self.nearest(position)
        {
            let shift = self.recenter(nearest, &element);
            drag.position += shift;
            position += shift;
        }
        self.scroll(&element, position, ScrollBehavior::Instant);
        if time - drag.sample.1 > FLICK_WINDOW {
            drag.sample = (point, time);
        }
        drag.last = (point, time);
        carousel.drag.set_value(Some(drag));
        event.prevent_default();
    }

    fn pointer_up(self, event: &ev::PointerEvent) {
        let (Some(carousel), Some(element)) = (self.carousel(), self.container_node()) else {
            return;
        };
        let Some(drag) = carousel.drag.get_value() else {
            return;
        };
        if event.pointer_id() != drag.pointer {
            return;
        }
        carousel.drag.set_value(None);
        if drag.captured {
            let _ = element.release_pointer_capture(drag.pointer);
        }
        if !drag.moved {
            return;
        }
        let options = carousel.options.get_untracked();
        let (max, rtl) = carousel
            .layout
            .with_value(|layout| (layout.max, layout.rtl));
        let time = event.time_stamp();
        let elapsed = time - drag.sample.1;
        let speed = (drag.last.0 - drag.sample.0) / elapsed;
        let flick = elapsed > 0.0 && time - drag.last.1 <= FLICK_WINDOW && speed.abs() > 0.1;
        let boost = if options.drag_free {
            FREE_BOOST
        } else {
            SNAP_BOOST
        };
        let force = match (flick, rtl) {
            (false, _) => 0.0,
            (true, false) => -speed * boost,
            (true, true) => speed * boost,
        };
        let position = self.position(&element);
        if options.drag_free {
            let destination = (position + force).clamp(0.0, max);
            let index = self.nearest(destination).unwrap_or(0);
            self.select(index);
            self.head(index, destination, ScrollBehavior::Smooth, &element);
            return;
        }
        let index = if force.abs() < NEXT_SNAP_FORCE {
            self.nearest(position + force)
        } else if options.skip_snaps && self.nearest(position) != Some(drag.index) {
            self.nearest(position + force * 0.5)
        } else {
            self.neighbour(drag.index, force > 0.0).or(Some(drag.index))
        };
        match index {
            Some(index) => self.go(index, ScrollBehavior::Smooth),
            None => self.settled(),
        }
    }

    fn clicked(self, event: &ev::MouseEvent) {
        let Some(carousel) = self.carousel() else {
            return;
        };
        if carousel.prevent_click.get_value() {
            carousel.prevent_click.set_value(false);
            event.stop_propagation();
            event.prevent_default();
        }
    }

    // shadcn's keys, whichever way the carousel runs
    fn key(self, event: &ev::KeyboardEvent) {
        match event.key().as_str() {
            "ArrowLeft" => {
                event.prevent_default();
                self.scroll_prev();
            }
            "ArrowRight" => {
                event.prevent_default();
                self.scroll_next();
            }
            _ => {}
        }
    }
}

// only what changes, so a slide the server styled keeps its style attribute as it came
fn put(element: &HtmlElement, name: &str, value: Option<&str>) -> bool {
    let style = element.style();
    let had = style.get_property_value(name).unwrap_or_default();
    match value {
        Some(value) if value != had => style.set_property(name, value).is_ok(),
        None if !had.is_empty() => style.remove_property(name).is_ok(),
        _ => false,
    }
}

fn slides(element: &web_sys::Element) -> Vec<HtmlElement> {
    let children = element.children();
    (0..children.length())
        .filter_map(|index| children.item(index))
        .filter_map(|child| child.dyn_into::<HtmlElement>().ok())
        .collect()
}

fn offset(align: Align, view: f64, size: f64) -> f64 {
    match align {
        Align::Start => 0.0,
        Align::Center => (view - size) / 2.0,
        Align::End => view - size,
    }
}

// slides as first and end of each group, end excluded
fn groups(by: SlidesToScroll, starts: &[f64], sizes: &[f64], view: f64) -> Vec<(usize, usize)> {
    let count = starts.len();
    match by {
        SlidesToScroll::Count(size) => (0..count)
            .step_by(size.max(1))
            .map(|first| (first, (first + size.max(1)).min(count)))
            .collect(),
        SlidesToScroll::Auto => {
            let mut groups = Vec::new();
            let mut first = 0;
            for index in 0..count {
                if index > first && starts[index] + sizes[index] - starts[first] > view + TOLERANCE
                {
                    groups.push((first, index));
                    first = index;
                }
            }
            if count > 0 {
                groups.push((first, count));
            }
            groups
        }
    }
}

// embla's containment: snaps held within the scroll, and with trim_snaps no repeats at the ends
fn contain(
    aligned: &[f64],
    max: f64,
    contain: ContainScroll,
) -> (Vec<f64>, std::ops::Range<usize>) {
    if max <= TOLERANCE {
        return (vec![0.0], 0..1);
    }
    let last = aligned.len() - 1;
    let bounded: Vec<f64> = aligned
        .iter()
        .enumerate()
        .map(|(index, snap)| {
            if index == 0 {
                return 0.0;
            }
            if index == last {
                return max;
            }
            let snap = snap.clamp(0.0, max);
            if snap < TOLERANCE {
                0.0
            } else if max - snap < TOLERANCE {
                max
            } else {
                (snap * 1000.0).round() / 1000.0
            }
        })
        .collect();
    match contain {
        ContainScroll::KeepSnaps => (bounded, 0..aligned.len()),
        ContainScroll::TrimSnaps => {
            let start = bounded.iter().rposition(|snap| *snap == 0.0).unwrap_or(0);
            let end = bounded
                .iter()
                .position(|snap| *snap == max)
                .map_or(bounded.len(), |index| index + 1);
            (bounded[start..end].to_vec(), start..end)
        }
    }
}

fn observe(api: CarouselApi, element: &HtmlElement) {
    let measure = Closure::<dyn Fn()>::new(move || api.remeasure());
    let Ok(resize) = web_sys::ResizeObserver::new(measure.as_ref().unchecked_ref()) else {
        return;
    };
    let watch = {
        let resize = resize.clone();
        let element = element.clone();
        move || {
            resize.disconnect();
            resize.observe(&element);
            for slide in slides(&element) {
                resize.observe(&slide);
            }
        }
    };
    watch();
    let changed = Closure::<dyn Fn()>::new(move || {
        watch();
        api.remeasure();
    });
    let Ok(mutation) = web_sys::MutationObserver::new(changed.as_ref().unchecked_ref()) else {
        return;
    };
    let init = web_sys::MutationObserverInit::new();
    init.set_child_list(true);
    let _ = mutation.observe_with_options(element, &init);

    // a wheel takes over from a scroll we started; passive, so the page keeps scrolling smoothly
    let wheel = Closure::<dyn Fn()>::new(move || api.interrupted());
    let options = web_sys::AddEventListenerOptions::new();
    options.set_passive(true);
    let _ = element.add_event_listener_with_callback_and_add_event_listener_options(
        "wheel",
        wheel.as_ref().unchecked_ref(),
        &options,
    );

    let held = StoredValue::new_local((element.clone(), resize, measure, mutation, changed, wheel));
    on_cleanup(move || {
        held.try_with_value(|(element, resize, _, mutation, _, wheel)| {
            resize.disconnect();
            mutation.disconnect();
            let _ = element
                .remove_event_listener_with_callback("wheel", wheel.as_ref().unchecked_ref());
        });
    });
}

// embla underneath: the content scrolls natively, snapping to its slides
#[component(element = "div")]
pub fn CarouselRoot(
    #[prop(into, default = Orientation::Horizontal.into())] orientation: Signal<Orientation>,
    #[prop(optional, into)] opts: Signal<CarouselOptions>,
    #[prop(optional)] plugins: Vec<AnyCarouselPlugin>,
    #[prop(optional_no_strip, into)] set_api: Option<Callback<CarouselApi>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let carousel = Carousel {
        orientation,
        options: opts,
        scroller: NodeRef::new(),
        list: RwSignal::new(Vec::new()),
        selected: RwSignal::new(0),
        previous: RwSignal::new(0),
        progress: RwSignal::new(0.0),
        looping: RwSignal::new(false),
        layout: StoredValue::new(Layout::default()),
        heading: StoredValue::new(None),
        drag: StoredValue::new(None),
        held: StoredValue::new(false),
        prevent_click: StoredValue::new(false),
        rotation: StoredValue::new(0),
        scrollend: StoredValue::new(true),
        settling: StoredValue::new(None),
    };
    let api = CarouselApi(StoredValue::new(carousel));

    Effect::new(move |_| {
        let Some(element) = carousel.scroller.get() else {
            return;
        };
        let element: HtmlElement = element.into();
        carousel.scrollend.set_value(
            web_sys::js_sys::Reflect::has(&window(), &"onscrollend".into()).unwrap_or(false),
        );
        api.init(&element);
        observe(api, &element);
        let plugins = plugins.clone();
        untrack(|| {
            for plugin in &plugins {
                plugin.0.init(api);
            }
            if let Some(set_api) = set_api {
                set_api.run(api);
            }
        });
        on_cleanup(move || {
            for plugin in &plugins {
                plugin.0.destroy();
            }
            if let Some(Some(handle)) = carousel.settling.try_update_value(Option::take) {
                handle.clear();
            }
        });
    });

    Effect::new(move |ran: Option<()>| {
        orientation.track();
        opts.track();
        if ran.is_some() {
            untrack(|| api.remeasure());
        }
    });

    view! {
        <div {..attrs} on:keydown:capture=move |event| api.key(&event)>
            <Provider value=api>{children.map(|children| children())}</Provider>
        </div>
    }
}

// for the element whose children are the slides, inside a CarouselRoot
pub fn carousel_container_attributes() -> impl Attribute {
    let api = use_carousel();
    let carousel = api
        .carousel()
        .expect("carousel_container_attributes runs inside a CarouselRoot");
    let orientation = carousel.orientation;
    let options = carousel.options;
    let overflow = move |axis: Orientation| {
        move || {
            if !options.with(|options| options.watch_drag) {
                "hidden"
            } else if orientation.get() == axis {
                "auto"
            } else {
                "hidden"
            }
        }
    };
    let attributes = view! {
        <{..}
            data-orientation=keyword(orientation)
            style:overflow-x=overflow(Orientation::Horizontal)
            style:overflow-y=overflow(Orientation::Vertical)
            style:scroll-snap-type=move || {
                if options.with(|options| options.drag_free) {
                    "none"
                } else if orientation.get() == Orientation::Vertical {
                    "y mandatory"
                } else {
                    "x mandatory"
                }
            }
            style:overscroll-behavior-x="contain"
            style:scrollbar-width="none"
            on:scroll=move |_| api.scrolled()
            on:scrollend=move |_| api.settled()
            on:pointerdown=move |event| api.pointer_down(&event)
            on:pointermove=move |event| api.pointer_move(&event)
            on:pointerup=move |event| api.pointer_up(&event)
            on:pointercancel=move |event| api.pointer_up(&event)
            on:dragstart=move |event: ev::DragEvent| {
                if options.with_untracked(|options| options.watch_drag) {
                    event.prevent_default();
                }
            }
            on:click:capture=move |event| api.clicked(&event)
        />
    };
    (attributes, node_ref(carousel.scroller))
}

// for each slide, a child of the element with carousel_container_attributes
pub fn carousel_slide_attributes() -> impl Attribute {
    let carousel = use_carousel()
        .carousel()
        .expect("carousel_slide_attributes runs inside a CarouselRoot");
    let options = carousel.options;
    view! {
        <{..}
            data-orientation=keyword(carousel.orientation)
            style:scroll-snap-align=move || options.with(|options| options.align.as_str())
            style:scroll-snap-stop=move || {
                if options.with(|options| options.skip_snaps) { "normal" } else { "always" }
            }
        />
    }
}
