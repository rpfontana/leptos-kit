use crate::dialog::{DialogContext, InitialFocus};
use crate::native::observe_resize;
use crate::swipe::{REVERSE_CANCEL_THRESHOLD, SwipeDirection, displacement, transform_of};
use crate::transition::{TransitionStatus, transition_attributes};
use crate::{ChangeReason, Controlled, Orientation};
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_kit_macros::component;
use send_wrapper::SendWrapper;
use web_sys::{Element, HtmlElement, HtmlInputElement, HtmlTextAreaElement};

const DEFAULT_SWIPE_THRESHOLD: f64 = 40.0;
const MIN_SWIPE_THRESHOLD: f64 = 10.0;
const FAST_SWIPE_VELOCITY: f64 = 0.5;
const SNAP_VELOCITY_THRESHOLD: f64 = 0.5;
const SNAP_VELOCITY_MULTIPLIER: f64 = 300.0;
const MAX_SNAP_VELOCITY: f64 = 4.0;
const MIN_RELEASE_VELOCITY: f64 = 0.2;
const MAX_RELEASE_VELOCITY: f64 = 4.0;
const MIN_RELEASE_MS: f64 = 80.0;
const MAX_RELEASE_MS: f64 = 360.0;
const MIN_RELEASE_SCALAR: f64 = 0.1;
const MAX_RELEASE_SCALAR: f64 = 1.0;
const AXIS_LOCK_SLOP: f64 = 6.0;
const AXIS_LOCK_BIAS: f64 = 2.0;
const MIN_VELOCITY_MS: f64 = 50.0;
const MIN_SAMPLE_MS: f64 = 16.0;
const MAX_SAMPLE_AGE_MS: f64 = 80.0;
const IGNORE: &str = "button,a,input,select,textarea,label,[role=\"button\"]";
const SWIPE_IGNORE: &str = "[data-base-ui-swipe-ignore]";
const CONTENT: &str = "[data-drawer-content]";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrawerSnapPoint {
    Fraction(f64),
    Px(f64),
    Rem(f64),
}

// past 1 a number is pixels
impl From<f64> for DrawerSnapPoint {
    fn from(value: f64) -> Self {
        if value <= 1.0 {
            Self::Fraction(value)
        } else {
            Self::Px(value)
        }
    }
}

impl DrawerSnapPoint {
    fn height(self, viewport: f64, rem: f64) -> Option<f64> {
        if !viewport.is_finite() || viewport <= 0.0 {
            return None;
        }
        let height = match self {
            Self::Fraction(fraction) => fraction.clamp(0.0, 1.0) * viewport,
            Self::Px(px) => px,
            Self::Rem(rems) => rems * rem,
        };
        height.is_finite().then_some(height)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Resolved {
    value: DrawerSnapPoint,
    height: f64,
    offset: f64,
}

fn resolve(points: &[DrawerSnapPoint], viewport: f64, popup: f64, rem: f64) -> Vec<Resolved> {
    if points.is_empty() || viewport <= 0.0 || popup <= 0.0 {
        return Vec::new();
    }
    let max = popup.min(viewport);
    let all = points.iter().filter_map(|&value| {
        let height = value.height(viewport, rem)?.clamp(0.0, max);
        Some(Resolved {
            value,
            height,
            offset: (popup - height).max(0.0),
        })
    });
    let all: Vec<Resolved> = all.collect();
    let mut kept: Vec<Resolved> = Vec::new();
    for point in all.into_iter().rev() {
        if !kept
            .iter()
            .any(|seen| (seen.height - point.height).abs() <= 1.0)
        {
            kept.push(point);
        }
    }
    kept.reverse();
    kept
}

fn closest(values: impl Iterator<Item = f64>, target: f64) -> Option<usize> {
    values
        .enumerate()
        .map(|(index, value)| (index, (value - target).abs()))
        .fold(
            None,
            |best: Option<(usize, f64)>, (index, distance)| match best {
                Some((_, nearest)) if nearest <= distance => best,
                _ => Some((index, distance)),
            },
        )
        .map(|(index, _)| index)
}

// math.sign: zero stays zero
fn sign(value: f64) -> f64 {
    if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else {
        0.0
    }
}

#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "client coordinates"
)]
fn element_at((x, y): (f64, f64)) -> Option<Element> {
    document().element_from_point(x as f32, y as f32)
}

fn vertical(direction: SwipeDirection) -> bool {
    matches!(direction, SwipeDirection::Up | SwipeDirection::Down)
}

fn axis_of(direction: SwipeDirection) -> Orientation {
    if vertical(direction) {
        Orientation::Vertical
    } else {
        Orientation::Horizontal
    }
}

fn size(element: &HtmlElement, direction: SwipeDirection) -> f64 {
    f64::from(if vertical(direction) {
        element.offset_height()
    } else {
        element.offset_width()
    })
}

fn threshold(element: &HtmlElement, direction: SwipeDirection) -> f64 {
    (size(element, direction) * 0.5).max(MIN_SWIPE_THRESHOLD)
}

fn snap_movement(base: f64, movement: f64) -> f64 {
    let next = base + movement;
    if next >= 0.0 {
        movement
    } else {
        -(-next).sqrt() - base
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Details {
    x: f64,
    y: f64,
    direction: Option<SwipeDirection>,
}

struct Release {
    direction: Option<SwipeDirection>,
    delta: (f64, f64),
    velocity: (f64, f64),
    last_velocity: (f64, f64),
}

#[derive(Clone)]
struct TouchScroll {
    start: (f64, f64),
    last: (f64, f64),
    target: Option<SendWrapper<HtmlElement>>,
    cross: bool,
    allow: Option<bool>,
    preserve: bool,
    attributed: bool,
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "one flag per step of the gesture"
)]
struct Gesture {
    dismissed: bool,
    start: (f64, f64),
    offset: (f64, f64),
    last_move: Option<(f64, f64)>,
    initial: (f64, f64, f64),
    intended: Option<SwipeDirection>,
    furthest: f64,
    cancelled: bool,
    baseline: (f64, f64),
    locked: Option<Orientation>,
    first_move: bool,
    pending: Option<(f64, f64)>,
    from_scrollable: bool,
    saw_primary: bool,
    size: (f64, f64),
    progress: f64,
    details: Option<Details>,
    threshold: f64,
    started_at: Option<f64>,
    sample: Option<(f64, f64, f64)>,
    velocity: (f64, f64),
    snapshot: Option<(String, String)>,
    pending_snap: Option<DrawerSnapPoint>,
    pointer_type: String,
    pen_touch: bool,
    ignore_touch: bool,
    touch: Option<TouchScroll>,
}

impl Default for Gesture {
    fn default() -> Self {
        Self {
            dismissed: false,
            start: (0.0, 0.0),
            offset: (0.0, 0.0),
            last_move: None,
            initial: (0.0, 0.0, 1.0),
            intended: None,
            furthest: 0.0,
            cancelled: false,
            baseline: (0.0, 0.0),
            locked: None,
            first_move: false,
            pending: None,
            from_scrollable: false,
            saw_primary: false,
            size: (0.0, 0.0),
            progress: 0.0,
            details: None,
            threshold: DEFAULT_SWIPE_THRESHOLD,
            started_at: None,
            sample: None,
            velocity: (0.0, 0.0),
            snapshot: None,
            pending_snap: None,
            pointer_type: String::new(),
            pen_touch: false,
            ignore_touch: false,
            touch: None,
        }
    }
}

impl Gesture {
    fn record(&mut self, offset: (f64, f64), time: Option<f64>) {
        let Some(time) = time else {
            return;
        };
        if let Some((x, y, at)) = self.sample
            && time > at
        {
            let ms = (time - at).max(MIN_SAMPLE_MS);
            self.velocity = ((offset.0 - x) / ms, (offset.1 - y) / ms);
        }
        self.sample = Some((offset.0, offset.1, time));
    }

    fn delta(&self) -> (f64, f64) {
        (
            self.offset.0 - self.initial.0,
            self.offset.1 - self.initial.1,
        )
    }
}

#[derive(Clone, Copy)]
enum Input<'a> {
    Pointer(&'a ev::PointerEvent),
    Touch(&'a web_sys::TouchEvent, &'a HtmlElement),
}

impl<'a> Input<'a> {
    fn as_event(self) -> &'a web_sys::Event {
        match self {
            Self::Pointer(event) => event.as_ref(),
            Self::Touch(event, _) => event.as_ref(),
        }
    }

    fn touch_like(self) -> bool {
        match self {
            Self::Pointer(event) => event.pointer_type() == "touch",
            Self::Touch(..) => true,
        }
    }

    fn position(self) -> Option<(f64, f64)> {
        match self {
            Self::Pointer(event) => {
                Some((f64::from(event.client_x()), f64::from(event.client_y())))
            }
            Self::Touch(event, _) => {
                let touch = event.touches().get(0)?;
                Some((f64::from(touch.client_x()), f64::from(touch.client_y())))
            }
        }
    }

    fn time(self) -> Option<f64> {
        let time = self.as_event().time_stamp();
        (time.is_finite() && time > 0.0).then_some(time)
    }

    fn target(self) -> Option<Element> {
        self.as_event()
            .target()
            .and_then(|target| target.dyn_into::<Element>().ok())
    }
}

#[derive(Clone, Copy)]
struct DrawerContext(StoredValue<Drawer>);

#[derive(Clone, Copy)]
struct Drawer {
    dialog: DialogContext,
    parent: Option<DrawerContext>,
    nested: RwSignal<Vec<DrawerContext>>,
    direction: Signal<SwipeDirection>,
    snap_points: Signal<Vec<DrawerSnapPoint>>,
    snap: Controlled<Option<DrawerSnapPoint>>,
    default_snap: Signal<Option<DrawerSnapPoint>>,
    sequential: Signal<bool>,
    popup_height: RwSignal<f64>,
    viewport_height: RwSignal<f64>,
    rem: RwSignal<f64>,
    resolved: Memo<Vec<Resolved>>,
    viewport: NodeRef<html::Div>,
    backdrop: NodeRef<html::Div>,
    swiping: RwSignal<bool>,
    swipe_active: RwSignal<bool>,
    progress: RwSignal<f64>,
    strength: RwSignal<Option<f64>>,
    // not thread-local: the server may drop it on another thread, which panics
    gesture: StoredValue<Gesture>,
}

impl DrawerContext {
    fn drawer(self) -> Drawer {
        self.0.get_value()
    }

    fn try_drawer(self) -> Option<Drawer> {
        self.0.try_get_value()
    }

    fn open(self) -> bool {
        self.try_drawer().is_some_and(|drawer| drawer.dialog.open())
    }

    fn present(self) -> bool {
        self.try_drawer().is_some_and(|drawer| {
            drawer.dialog.open() || drawer.dialog.phase.get() == TransitionStatus::Ending
        })
    }

    fn children(self) -> Vec<DrawerContext> {
        self.try_drawer()
            .map(|drawer| drawer.nested.get())
            .unwrap_or_default()
    }

    fn nested_open(self) -> usize {
        self.children()
            .into_iter()
            .filter(|child| child.open())
            .map(|child| 1 + child.nested_open())
            .max()
            .unwrap_or(0)
    }

    fn has_nested(self) -> bool {
        self.children().into_iter().any(DrawerContext::present)
    }

    fn nested_swiping(self) -> bool {
        self.children().into_iter().any(|child| {
            child
                .try_drawer()
                .is_some_and(|drawer| drawer.swipe_active.get())
                || child.nested_swiping()
        })
    }

    fn nested_progress(self) -> f64 {
        self.children()
            .into_iter()
            .map(|child| {
                child
                    .try_drawer()
                    .map_or(0.0, |drawer| drawer.progress.get())
                    .max(child.nested_progress())
            })
            .fold(0.0, f64::max)
    }

    fn frontmost(self) -> f64 {
        let nested = self
            .children()
            .into_iter()
            .filter(|child| child.open())
            .map(DrawerContext::frontmost)
            .find(|height| *height > 0.0);
        nested.unwrap_or_else(|| {
            self.try_drawer()
                .map_or(0.0, |drawer| drawer.popup_height.get())
        })
    }

    fn has_snap_points(self) -> bool {
        self.try_drawer()
            .is_some_and(|drawer| drawer.snap_points.with(|points| !points.is_empty()))
    }

    fn active_snap(self) -> Option<DrawerSnapPoint> {
        let drawer = self.try_drawer()?;
        let active = drawer.snap.get();
        if drawer.snap.is_controlled() || !self.has_snap_points() {
            return active;
        }
        match active {
            Some(point) if drawer.snap_points.with(|points| points.contains(&point)) => Some(point),
            _ => drawer.default_snap.get(),
        }
    }

    fn active_offset(self) -> Option<f64> {
        let drawer = self.try_drawer()?;
        let active = self.active_snap()?;
        drawer.resolved.with(|points| {
            if let Some(exact) = points.iter().find(|point| point.value == active) {
                return Some(exact.offset);
            }
            let viewport = drawer.viewport_height.get();
            let height = active
                .height(viewport, drawer.rem.get())?
                .clamp(0.0, drawer.popup_height.get().min(viewport).max(0.0));
            closest(points.iter().map(|point| point.height), height).map(|at| points[at].offset)
        })
    }

    fn expanded(self) -> bool {
        self.active_snap() == Some(DrawerSnapPoint::Fraction(1.0))
    }

    fn directions(self) -> Vec<SwipeDirection> {
        let Some(drawer) = self.try_drawer() else {
            return Vec::new();
        };
        let direction = drawer.direction.get();
        match direction {
            SwipeDirection::Down if self.has_snap_points() => {
                vec![SwipeDirection::Down, SwipeDirection::Up]
            }
            SwipeDirection::Up if self.has_snap_points() => {
                vec![SwipeDirection::Up, SwipeDirection::Down]
            }
            _ => vec![direction],
        }
    }

    fn snap_range(self) -> Option<(f64, f64)> {
        let drawer = self.try_drawer()?;
        let points = drawer.snap_points.with(Vec::len);
        let mut offsets: Vec<f64> = drawer
            .resolved
            .with(|resolved| resolved.iter().map(|point| point.offset).collect());
        if points < 2 || offsets.len() < 2 || !vertical(drawer.direction.get()) {
            return None;
        }
        offsets.sort_by(f64::total_cmp);
        Some((offsets[0], offsets[1] - offsets[0]))
    }

    fn snap_progress(self) -> Option<f64> {
        let (min, range) = self.snap_range()?;
        let offset = self.active_offset()?;
        Some(((offset - min) / range).clamp(0.0, 1.0))
    }

    fn set_snap(self, point: Option<DrawerSnapPoint>) {
        if let Some(drawer) = self.try_drawer() {
            drawer.snap.set(point, ChangeReason::Swipe);
        }
    }

    fn enabled(self) -> bool {
        untrack(|| self.present() && self.nested_open() == 0)
    }

    fn popup(self) -> Option<HtmlElement> {
        let popup = self.try_drawer()?.dialog.popup.get_untracked()?;
        Some(popup.into())
    }

    fn set_swiping(self, swiping: bool) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        if drawer.swiping.get_untracked() == swiping {
            return;
        }
        drawer.swiping.set(swiping);
        if let Some(backdrop) = drawer.backdrop.get_untracked() {
            let _ = backdrop.toggle_attribute_with_force("data-swiping", swiping);
        }
        if !swiping && drawer.parent.is_none() {
            self.finish_nested_swipe();
        }
    }

    fn finish_nested_swipe(self) {
        if let Some(drawer) = self.try_drawer()
            && drawer.swipe_active.get_untracked()
        {
            drawer.swipe_active.set(false);
        }
    }

    fn set_dismissed(self, dismissed: bool) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        if let Some(popup) = drawer.dialog.popup.get_untracked() {
            let _ = popup.toggle_attribute_with_force("data-swipe-dismiss", dismissed);
        }
        if let Some(backdrop) = drawer.backdrop.get_untracked() {
            let _ = backdrop.toggle_attribute_with_force("data-swipe-dismiss", dismissed);
        }
    }

    fn clear_release(self) {
        self.set_dismissed(false);
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        if let Some(popup) = drawer.dialog.popup.get_untracked() {
            let _ = popup.remove_attribute("data-ending-style");
        }
        drawer.strength.set(None);
    }

    fn sync_drag(self, swiping: bool) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        let Some(popup) = self.popup() else {
            if !swiping {
                drawer
                    .gesture
                    .update_value(|gesture| gesture.snapshot = None);
            }
            return;
        };
        let style = popup.style();
        if swiping {
            drawer.gesture.update_value(|gesture| {
                if gesture.snapshot.is_none() {
                    gesture.snapshot = Some((
                        style.get_property_value("transition").unwrap_or_default(),
                        style.get_property_value("transform").unwrap_or_default(),
                    ));
                }
            });
            let _ = style.set_property("transition", "none");
        } else if let Some(Some((transition, transform))) = drawer
            .gesture
            .try_update_value(|gesture| gesture.snapshot.take())
        {
            let _ = style.set_property("transition", &transition);
            let _ = style.set_property("transform", &transform);
        }
        let Some(((x, y), offset, scale)) = drawer
            .gesture
            .try_with_value(|gesture| (gesture.delta(), gesture.offset, gesture.initial.2))
        else {
            return;
        };
        if swiping {
            let _ = style.set_property(
                "transform",
                &format!(
                    "translate3d({}px,{}px,0) scale({scale})",
                    offset.0, offset.1
                ),
            );
        }
        let _ = style.set_property("--drawer-swipe-movement-x", &format!("{x}px"));
        let _ = style.set_property("--drawer-swipe-movement-y", &format!("{y}px"));
    }

    fn reset(self) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        self.set_swiping(false);
        self.update_progress(0.0, None);
        drawer.gesture.update_value(|gesture| {
            *gesture = Gesture {
                snapshot: gesture.snapshot.take(),
                pending_snap: gesture.pending_snap.take(),
                pointer_type: std::mem::take(&mut gesture.pointer_type),
                pen_touch: gesture.pen_touch,
                ignore_touch: gesture.ignore_touch,
                touch: gesture.touch.take(),
                ..Gesture::default()
            };
        });
        self.sync_drag(false);
    }

    fn update_progress(self, progress: f64, details: Option<Details>) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        let progress = if progress.is_finite() {
            progress.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let changed = drawer.gesture.try_update_value(|gesture| {
            let moved = progress != gesture.progress;
            let other = details.is_some_and(|details| gesture.details != Some(details));
            if !moved && !other {
                return false;
            }
            gesture.progress = progress;
            if details.is_some() {
                gesture.details = details;
            } else if moved {
                gesture.details = None;
            }
            true
        });
        if changed == Some(true) {
            self.on_progress(progress, details);
        }
    }

    fn mark_nested_swipe(self, details: Option<Details>) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        let Some(details) = details else {
            return;
        };
        if drawer.swipe_active.get_untracked() {
            return;
        }
        let direction = details
            .direction
            .unwrap_or_else(|| drawer.direction.get_untracked());
        if displacement(direction, details.x, details.y).abs() >= MIN_SWIPE_THRESHOLD {
            drawer.swipe_active.set(true);
        }
    }

    fn on_progress(self, progress: f64, details: Option<Details>) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        let swiping = drawer.swiping.get_untracked();
        if swiping {
            self.mark_nested_swipe(details);
        }
        let (direction, snaps, offset, popup_height) = untrack(|| {
            (
                drawer.direction.get(),
                self.has_snap_points(),
                self.active_offset(),
                drawer.popup_height.get(),
            )
        });
        if swiping
            && direction == SwipeDirection::Down
            && snaps
            && let Some(details) = details
            && let Some(popup) = self.popup()
        {
            let style = popup.style();
            let _ = style.remove_property("transform");
            let movement = snap_movement(offset.unwrap_or(0.0), details.y);
            let _ = style.set_property("--drawer-swipe-movement-y", &format!("{movement}px"));
        }
        let mut resolved = progress;
        if let Some((min, range)) = untrack(|| self.snap_range())
            && popup_height > 0.0
        {
            let base = offset.unwrap_or(min);
            match details {
                Some(details) if swiping && details.y.is_finite() => {
                    let next = (base + details.y).clamp(0.0, popup_height);
                    resolved = ((next - min) / range).clamp(0.0, 1.0);
                }
                _ => {
                    if let Some(progress) = untrack(|| self.snap_progress()) {
                        resolved = progress;
                    }
                }
            }
        }
        // a parent follows a drag only, so it drops back once the drag ends
        if !swiping {
            if drawer.parent.is_some() {
                drawer.progress.set(0.0);
            }
            self.finish_nested_swipe();
        }
        self.apply_progress(resolved, swiping);
    }

    fn apply_progress(self, progress: f64, notify_parent: bool) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        let open = untrack(|| drawer.dialog.open());
        let nested = drawer.parent.is_some();
        let own = if open && !nested { progress } else { 0.0 };
        let reported = if open { progress } else { 0.0 };
        if notify_parent && nested {
            drawer.progress.set(reported);
            if reported <= 0.0 {
                self.finish_nested_swipe();
            }
        }
        let Some(backdrop) = drawer.backdrop.get_untracked() else {
            return;
        };
        let style = HtmlElement::style(&backdrop);
        let shown = own > 0.0;
        let value = if shown { own.to_string() } else { "0".into() };
        let _ = style.set_property("--drawer-swipe-progress", &value);
        let frontmost = untrack(|| self.frontmost());
        if shown && frontmost > 0.0 {
            let _ = style.set_property("--drawer-height", &format!("{frontmost}px"));
        } else {
            let _ = style.remove_property("--drawer-height");
        }
    }

    fn can_start(self, at: (f64, f64), touch_like: bool) -> bool {
        let Some(popup) = self.popup() else {
            return false;
        };
        let inside = element_at(at).is_some_and(|element| popup.contains(Some(&element)));
        inside && !(touch_like && selecting(&popup))
    }

    fn axes(self) -> (bool, bool) {
        let directions = untrack(|| self.directions());
        (
            directions.iter().any(|direction| !vertical(*direction)),
            directions.iter().any(|direction| vertical(*direction)),
        )
    }

    fn allows(self, direction: SwipeDirection) -> bool {
        untrack(|| self.directions()).contains(&direction)
    }

    fn primary(self) -> Option<SwipeDirection> {
        match untrack(|| self.directions()).as_slice() {
            [only] => Some(*only),
            _ => None,
        }
    }

    fn start(self, input: Input<'_>) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        if !self.enabled() || input.as_event().default_prevented() {
            return;
        }
        if let Input::Pointer(event) = input
            && event.button() != 0
        {
            return;
        }
        let Some(at) = input.position() else {
            return;
        };
        drawer.gesture.update_value(|gesture| {
            gesture.pending = Some(at);
            gesture.from_scrollable = false;
            gesture.saw_primary = matches!(input, Input::Pointer(_));
        });
        if !self.can_start(at, input.touch_like()) {
            return;
        }
        if self.begin(input, at, false) {
            drawer
                .gesture
                .update_value(|gesture| gesture.pending = None);
        }
    }

    fn begin(self, input: Input<'_>, at: (f64, f64), from_edge: bool) -> bool {
        let Some(drawer) = self.try_drawer() else {
            return false;
        };
        let Some(popup) = self.popup() else {
            return false;
        };
        let touch_like = input.touch_like();
        let (has_horizontal, has_vertical) = self.axes();
        let target = element_at(at).or_else(|| input.target());
        let scrollable = if touch_like {
            target.as_ref().and_then(|target| {
                gesture_scroll_target(
                    target,
                    document().body()?.as_ref(),
                    has_horizontal,
                    has_vertical,
                )
            })
        } else {
            None
        };
        if scrollable.is_some() && !from_edge {
            return false;
        }
        if !touch_like
            && target
                .as_ref()
                .is_some_and(|target| target.closest(IGNORE).ok().flatten().is_some())
        {
            return false;
        }
        if !from_edge
            && target.as_ref().is_some_and(|target| {
                scrollable_ancestor(target, &popup, has_horizontal, has_vertical)
            })
        {
            return false;
        }
        let transform = transform_of(&popup);
        let threshold = self.primary().map_or(DEFAULT_SWIPE_THRESHOLD, |direction| {
            threshold(&popup, direction)
        });
        let size = (
            f64::from(popup.offset_width()),
            f64::from(popup.offset_height()),
        );
        let time = input.time();
        drawer.gesture.update_value(|gesture| {
            gesture.from_scrollable = scrollable.is_some() && from_edge;
            gesture.cancelled = false;
            gesture.intended = None;
            gesture.furthest = 0.0;
            gesture.start = at;
            gesture.started_at = time;
            gesture.baseline = at;
            gesture.last_move = Some(at);
            gesture.threshold = threshold;
            gesture.size = size;
            gesture.initial = transform;
            gesture.offset = (transform.0, transform.1);
            gesture.sample = None;
            gesture.record((transform.0, transform.1), time);
            gesture.locked = None;
            gesture.first_move = true;
        });
        if let Input::Pointer(event) = input {
            let _ = popup.set_pointer_capture(event.pointer_id());
        }
        if !touch_like {
            clear_selection(&popup);
        }
        self.set_swiping(true);
        self.update_progress(0.0, None);
        self.sync_drag(true);
        true
    }

    fn cancel(self, event: &ev::PointerEvent) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        drawer.gesture.update_value(|gesture| {
            gesture.pending = None;
            gesture.from_scrollable = false;
            gesture.last_move = None;
        });
        if !drawer.swiping.get_untracked() {
            return;
        }
        self.set_swiping(false);
        drawer.gesture.update_value(|gesture| {
            gesture.locked = None;
            gesture.offset = (gesture.initial.0, gesture.initial.1);
            gesture.saw_primary = false;
        });
        self.sync_drag(false);
        if let Some(popup) = self.popup() {
            let _ = popup.release_pointer_capture(event.pointer_id());
        }
        self.update_progress(
            0.0,
            Some(Details {
                x: 0.0,
                y: 0.0,
                direction: None,
            }),
        );
    }

    fn moved(self, input: Input<'_>) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        let Some(at) = input.position() else {
            return;
        };
        let mut end_after = false;
        if let Input::Pointer(event) = input {
            let buttons = event.buttons();
            let primary = buttons % 2 == 1;
            if primary {
                drawer
                    .gesture
                    .update_value(|gesture| gesture.saw_primary = true);
            }
            if buttons != 0 && !primary {
                self.cancel(event);
                return;
            }
            // a release can land as a move without buttons just before the pointerup
            if buttons == 0 && drawer.gesture.with_value(|gesture| gesture.saw_primary) {
                if !drawer.swiping.get_untracked() {
                    self.end(input);
                    return;
                }
                end_after = true;
            }
        }
        let pending = drawer.gesture.with_value(|gesture| gesture.pending);
        if !drawer.swiping.get_untracked()
            && let Some(start) = pending
        {
            if !input.touch_like() && input.as_event().default_prevented() {
                drawer.gesture.update_value(|gesture| {
                    gesture.pending = None;
                    gesture.from_scrollable = false;
                    gesture.last_move = None;
                });
                return;
            }
            if self.can_start(at, input.touch_like()) {
                let mut from_edge = false;
                if input.touch_like()
                    && let Some(popup) = self.popup()
                {
                    let (has_horizontal, has_vertical) = self.axes();
                    let target = element_at(at).or_else(|| input.target());
                    let scroll = target.and_then(|target| {
                        gesture_scroll_target(
                            &target,
                            document().body()?.as_ref(),
                            has_horizontal,
                            has_vertical,
                        )
                    });
                    if let Some(scroll) = scroll
                        && (popup.contains(Some(&scroll)) || scroll.contains(Some(&popup)))
                    {
                        match self.edge_on_pending(&scroll, (at.0 - start.0, at.1 - start.1)) {
                            Some(false) => return,
                            Some(true) => from_edge = true,
                            None => {}
                        }
                    }
                }
                if self.begin(input, at, from_edge) {
                    drawer.gesture.update_value(|gesture| {
                        gesture.pending = None;
                        if from_edge {
                            gesture.start = start;
                            gesture.baseline = start;
                            gesture.last_move = Some(start);
                            gesture.first_move = false;
                        } else {
                            gesture.from_scrollable = false;
                        }
                    });
                }
            }
        }
        let movement = drawer.gesture.with_value(|gesture| {
            gesture
                .last_move
                .map_or((0.0, 0.0), |last| (at.0 - last.0, at.1 - last.1))
        });
        drawer
            .gesture
            .update_value(|gesture| gesture.last_move = Some(at));
        self.move_core(input, at, movement);
        if end_after {
            self.end(input);
        }
    }

    fn edge_on_pending(self, scroll: &HtmlElement, (x, y): (f64, f64)) -> Option<bool> {
        let (has_horizontal, has_vertical) = self.axes();
        let can = |delta: f64, offset: f64, max: f64, toward_start: bool, toward_end: bool| {
            (delta > 0.0 && offset <= 0.0 && toward_start)
                || (delta < 0.0 && offset >= max.max(0.0) && toward_end)
        };
        if has_vertical && y != 0.0 && (!has_horizontal || y.abs() >= x.abs()) {
            return Some(can(
                y,
                f64::from(scroll.scroll_top()),
                f64::from(scroll.scroll_height() - scroll.client_height()),
                self.allows(SwipeDirection::Down),
                self.allows(SwipeDirection::Up),
            ));
        }
        if has_horizontal && x != 0.0 && (!has_vertical || x.abs() > y.abs()) {
            return Some(can(
                x,
                f64::from(scroll.scroll_left()),
                f64::from(scroll.scroll_width() - scroll.client_width()),
                self.allows(SwipeDirection::Right),
                self.allows(SwipeDirection::Left),
            ));
        }
        None
    }

    fn move_core(self, input: Input<'_>, at: (f64, f64), movement: (f64, f64)) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        if !self.enabled() || !drawer.swiping.get_untracked() {
            return;
        }
        let (has_horizontal, has_vertical) = self.axes();
        if let Input::Touch(_, root) = input
            && !drawer.gesture.with_value(|gesture| gesture.from_scrollable)
            && input.target().is_some_and(|target| {
                gesture_scroll_target(&target, root, has_horizontal, has_vertical).is_some()
            })
        {
            return;
        }
        if let Input::Pointer(event) = input {
            // safari would select text
            event.prevent_default();
        }
        let time = input.time();
        let (allow_left, allow_right) = (
            self.allows(SwipeDirection::Left),
            self.allows(SwipeDirection::Right),
        );
        let (allow_up, allow_down) = (
            self.allows(SwipeDirection::Up),
            self.allows(SwipeDirection::Down),
        );
        let popup = self.popup();
        let primary = self.primary();
        let Some((progress, details, changed)) = drawer.gesture.try_update_value(|gesture| {
            if gesture.first_move {
                gesture.first_move = false;
                gesture.start = at;
                if time.is_some() {
                    gesture.started_at = time;
                }
            }
            let (x, y) = at;
            if (movement.1 < 0.0 && y > gesture.baseline.1)
                || (movement.1 > 0.0 && y < gesture.baseline.1)
            {
                gesture.baseline.1 = y;
            }
            if (movement.0 < 0.0 && x > gesture.baseline.0)
                || (movement.0 > 0.0 && x < gesture.baseline.0)
            {
                gesture.baseline.0 = x;
            }
            let (delta_x, delta_y) = (x - gesture.start.0, y - gesture.start.1);
            let (cancel_x, cancel_y) = (x - gesture.baseline.0, y - gesture.baseline.1);
            if gesture.locked.is_none()
                && has_horizontal
                && has_vertical
                && delta_x.hypot(delta_y) >= 1.0
            {
                gesture.locked = Some(if delta_x.abs() > delta_y.abs() {
                    Orientation::Horizontal
                } else {
                    Orientation::Vertical
                });
            }
            if let Some(direction) = gesture.intended {
                let current = displacement(direction, cancel_x, cancel_y);
                if current > gesture.threshold {
                    gesture.cancelled = false;
                } else if !(allow_left && allow_right || allow_up && allow_down)
                    && gesture.furthest - current >= REVERSE_CANCEL_THRESHOLD
                {
                    gesture.cancelled = true;
                }
            } else {
                let candidate = match gesture.locked {
                    Some(Orientation::Vertical) if delta_y > 0.0 => Some(SwipeDirection::Down),
                    Some(Orientation::Vertical) if delta_y < 0.0 => Some(SwipeDirection::Up),
                    Some(Orientation::Horizontal) if delta_x > 0.0 => Some(SwipeDirection::Right),
                    Some(Orientation::Horizontal) if delta_x < 0.0 => Some(SwipeDirection::Left),
                    Some(_) => None,
                    None if delta_x.abs() >= delta_y.abs() => Some(if delta_x > 0.0 {
                        SwipeDirection::Right
                    } else {
                        SwipeDirection::Left
                    }),
                    None => Some(if delta_y > 0.0 {
                        SwipeDirection::Down
                    } else {
                        SwipeDirection::Up
                    }),
                };
                let allowed = candidate.filter(|candidate| match candidate {
                    SwipeDirection::Left => allow_left,
                    SwipeDirection::Right => allow_right,
                    SwipeDirection::Up => allow_up,
                    SwipeDirection::Down => allow_down,
                });
                if let Some(candidate) = allowed {
                    gesture.intended = Some(candidate);
                    gesture.furthest = displacement(candidate, delta_x, delta_y);
                    if let Some(popup) = &popup {
                        gesture.threshold = threshold(popup, candidate).max(0.0);
                    }
                }
            }
            let root = |value: f64| value.signum() * value.abs().sqrt();
            let damp = |delta: f64, negative: bool, positive: bool| {
                if (!negative && delta < 0.0) || (!positive && delta > 0.0) {
                    root(delta)
                } else {
                    delta
                }
            };
            let damped_x = if has_horizontal {
                damp(delta_x, allow_left, allow_right)
            } else {
                root(delta_x)
            };
            let damped_y = if has_vertical {
                damp(delta_y, allow_up, allow_down)
            } else {
                root(delta_y)
            };
            let (from_x, from_y, scale) = gesture.initial;
            let mut next = (from_x, from_y);
            if gesture.locked != Some(Orientation::Vertical) && has_horizontal {
                next.0 += damped_x;
            }
            if gesture.locked != Some(Orientation::Horizontal) && has_vertical {
                next.1 += damped_y;
            }
            let changed = next != gesture.offset;
            gesture.offset = next;
            gesture.record(next, time);
            let (drag_x, drag_y) = gesture.delta();
            let details = Details {
                x: drag_x,
                y: drag_y,
                direction: gesture.intended,
            };
            let mut progress = 0.0;
            if let Some(direction) = primary.or(gesture.intended) {
                let size = if vertical(direction) {
                    gesture.size.1
                } else {
                    gesture.size.0
                };
                let moved = displacement(direction, drag_x, drag_y);
                if size > 0.0 && scale > 0.0 && moved > 0.0 {
                    progress = moved / (size * scale);
                }
            }
            (progress, details, changed)
        }) else {
            return;
        };
        if changed {
            self.sync_drag(true);
        }
        self.update_progress(progress, Some(details));
    }

    fn end(self, input: Input<'_>) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        if !self.enabled() {
            return;
        }
        let Some((delta, intended)) = drawer
            .gesture
            .try_with_value(|gesture| (gesture.delta(), gesture.intended))
        else {
            return;
        };
        let details = Details {
            x: delta.0,
            y: delta.1,
            direction: intended,
        };
        let stop_pending = |gesture: &mut Gesture| {
            gesture.pending = None;
            gesture.from_scrollable = false;
            gesture.last_move = None;
        };
        if !drawer.swiping.get_untracked() {
            drawer.gesture.update_value(stop_pending);
            self.update_progress(0.0, Some(details));
            return;
        }
        self.set_swiping(false);
        drawer.gesture.update_value(|gesture| {
            gesture.locked = None;
            stop_pending(gesture);
            gesture.saw_primary = false;
        });
        if let Input::Pointer(event) = input
            && let Some(popup) = self.popup()
        {
            let _ = popup.release_pointer_capture(event.pointer_id());
        }
        let end = input.time();
        let Some((velocity, last_velocity, cancelled)) = drawer.gesture.try_with_value(|gesture| {
            let duration = match (gesture.started_at, end) {
                (Some(start), Some(end)) if end > start => end - start,
                _ => 0.0,
            };
            let ms = if duration > 0.0 {
                duration.max(MIN_VELOCITY_MS)
            } else {
                0.0
            };
            let velocity = if ms > 0.0 {
                (delta.0 / ms, delta.1 / ms)
            } else {
                (0.0, 0.0)
            };
            let mut last = gesture.velocity;
            if let (Some((x, y, at)), Some(end)) = (gesture.sample, end)
                && end >= at
            {
                let age = end - at;
                if age <= MAX_SAMPLE_AGE_MS {
                    let ms = age.max(MIN_SAMPLE_MS);
                    let sample = ((gesture.offset.0 - x) / ms, (gesture.offset.1 - y) / ms);
                    if sample.0 != 0.0 {
                        last.0 = sample.0;
                    }
                    if sample.1 != 0.0 {
                        last.1 = sample.1;
                    }
                } else {
                    last = (0.0, 0.0);
                }
            }
            (velocity, last, gesture.cancelled)
        }) else {
            return;
        };
        let decision = self.release(&Release {
            direction: intended,
            delta,
            velocity,
            last_velocity,
        });
        let restore = move || {
            drawer.gesture.update_value(|gesture| {
                gesture.offset = (gesture.initial.0, gesture.initial.1);
            });
            self.sync_drag(false);
            self.update_progress(0.0, Some(details));
        };
        if cancelled && decision.is_none() {
            restore();
            return;
        }
        let close = if let Some(close) = decision {
            close && intended.or_else(|| self.primary()).is_some()
        } else {
            let threshold = drawer.gesture.with_value(|gesture| gesture.threshold);
            untrack(|| self.directions())
                .into_iter()
                .any(|direction| displacement(direction, delta.0, delta.1) > threshold)
        };
        if close {
            drawer
                .gesture
                .update_value(|gesture| gesture.dismissed = true);
            self.sync_drag(false);
            self.dismiss();
        } else {
            restore();
        }
    }

    fn release_strength(
        self,
        popup: &HtmlElement,
        direction: SwipeDirection,
        release: &Release,
    ) -> Option<f64> {
        let size = size(popup, direction);
        if size <= 0.0 {
            return None;
        }
        let base = if vertical(direction) && untrack(|| self.has_snap_points()) {
            untrack(|| self.active_offset()).unwrap_or(0.0)
        } else {
            0.0
        };
        let travelled = base + displacement(direction, release.delta.0, release.delta.1);
        let remaining = (size - travelled).max(0.0);
        if remaining <= 0.0 {
            return None;
        }
        let last = displacement(direction, release.last_velocity.0, release.last_velocity.1);
        let velocity = if last.abs() > 0.0 {
            last
        } else {
            displacement(direction, release.velocity.0, release.velocity.1)
        };
        if velocity <= MIN_RELEASE_VELOCITY {
            return None;
        }
        let velocity = velocity.clamp(MIN_RELEASE_VELOCITY, MAX_RELEASE_VELOCITY);
        let ms = (remaining / velocity).clamp(MIN_RELEASE_MS, MAX_RELEASE_MS);
        let normalized = (ms - MIN_RELEASE_MS) / (MAX_RELEASE_MS - MIN_RELEASE_MS);
        Some(MIN_RELEASE_SCALAR + normalized * (MAX_RELEASE_SCALAR - MIN_RELEASE_SCALAR))
    }

    fn release(self, release: &Release) -> Option<bool> {
        let drawer = self.try_drawer()?;
        let Some(popup) = self.popup() else {
            self.clear_release();
            return None;
        };
        // the ending style starts at once, so the popup doesn't sit still before it leaves
        let leave = |direction: SwipeDirection| {
            self.finish_nested_swipe();
            self.set_dismissed(true);
            let _ = popup.style().remove_property("transition");
            let _ = popup.set_attribute("data-ending-style", "");
            drawer
                .strength
                .set(self.release_strength(&popup, direction, release));
        };
        let (dx, dy) = release.delta;
        if !untrack(|| self.has_snap_points()) {
            let Some(direction) = release.direction else {
                self.clear_release();
                return None;
            };
            let delta = displacement(direction, dx, dy);
            if delta <= 0.0 {
                self.clear_release();
                return Some(false);
            }
            if displacement(direction, release.velocity.0, release.velocity.1)
                >= FAST_SWIPE_VELOCITY
            {
                leave(direction);
                return Some(true);
            }
            let close = delta > threshold(&popup, direction);
            if close {
                leave(direction);
            } else {
                self.clear_release();
            }
            return Some(close);
        }
        let direction = drawer.direction.get_untracked();
        if !vertical(direction) {
            self.clear_release();
            return None;
        }
        let height = drawer.popup_height.get_untracked();
        if height == 0.0 {
            self.clear_release();
            return Some(false);
        }
        let resolved = drawer.resolved.get_untracked();
        if resolved.is_empty() {
            self.clear_release();
            return None;
        }
        let along = if direction == SwipeDirection::Down {
            1.0
        } else {
            -1.0
        };
        let drag = along * dy;
        let drag_sign = sign(drag);
        let mut velocity = along * release.last_velocity.1;
        if drag_sign != 0.0 && drag.abs() >= MIN_SWIPE_THRESHOLD {
            let velocity_sign = sign(velocity);
            if velocity_sign != 0.0 && velocity_sign != drag_sign {
                velocity = along * release.velocity.1;
            }
        }
        let current = untrack(|| self.active_offset()).unwrap_or(0.0);
        let dragged = (current + drag).clamp(0.0, height);
        let thrown = if velocity.abs() >= SNAP_VELOCITY_THRESHOLD {
            velocity.clamp(-MAX_SNAP_VELOCITY, MAX_SNAP_VELOCITY) * SNAP_VELOCITY_MULTIPLIER
        } else {
            0.0
        };
        let sequential = drawer.sequential.get_untracked();
        let target = if sequential {
            dragged
        } else {
            (dragged + thrown).clamp(0.0, height)
        };
        let settle = |point: Option<Resolved>| {
            if let Some(point) = point {
                self.set_snap(Some(point.value));
            }
            self.apply_progress(0.0, true);
            self.clear_release();
            Some(false)
        };
        let close = |fallback: Resolved| {
            if release.direction.is_none() {
                return settle(Some(fallback));
            }
            let active = untrack(|| self.active_snap());
            self.set_snap(None);
            drawer
                .gesture
                .update_value(|gesture| gesture.pending_snap = active);
            leave(direction);
            Some(true)
        };
        if sequential {
            let mut ordered = resolved;
            ordered.sort_by(|a, b| a.offset.total_cmp(&b.offset));
            let offsets = || ordered.iter().map(|point| point.offset);
            let current_index = closest(offsets(), current).unwrap_or(0);
            let mut point = ordered[closest(offsets(), target).unwrap_or(0)];
            let velocity_sign = sign(velocity);
            let advance = drag_sign != 0.0
                && velocity_sign != 0.0
                && velocity_sign == drag_sign
                && velocity.abs() >= SNAP_VELOCITY_THRESHOLD;
            let mut effective = target;
            if advance {
                let adjacent = if drag_sign > 0.0 {
                    (current_index + 1).min(ordered.len() - 1)
                } else {
                    current_index.saturating_sub(1)
                };
                if adjacent != current_index {
                    let next = ordered[adjacent];
                    let force = if drag_sign > 0.0 {
                        target < next.offset
                    } else {
                        target > next.offset
                    };
                    if force {
                        point = next;
                        effective = next.offset;
                    }
                } else if drag_sign > 0.0 {
                    return close(point);
                }
            }
            if (effective - height).abs() < (effective - point.offset).abs() {
                return close(point);
            }
            return settle(Some(point));
        }
        let at = closest(resolved.iter().map(|point| point.offset), target).unwrap_or(0);
        let point = resolved[at];
        if velocity >= FAST_SWIPE_VELOCITY && drag > 0.0 {
            return close(point);
        }
        if (target - height).abs() < (target - point.offset).abs() {
            return close(point);
        }
        settle(Some(point))
    }

    fn dismiss(self) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        if let Some(backdrop) = drawer.backdrop.get_untracked() {
            let style = HtmlElement::style(&backdrop);
            let _ = style.set_property("--drawer-swipe-progress", "0");
            let _ = style.remove_property("--drawer-height");
        }
        drawer.dialog.native.state().set(false, ChangeReason::Swipe);
        let pending = drawer
            .gesture
            .try_update_value(|gesture| gesture.pending_snap.take())
            .flatten();
        // a controlled drawer whose owner kept it open goes back where it was
        if drawer.dialog.native.state().get_untracked() {
            if pending.is_some() {
                self.set_snap(pending);
            }
            self.clear_release();
            self.reset();
            return;
        }
        self.set_dismissed(true);
    }

    fn touch_start(self, event: &web_sys::TouchEvent) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        let stop = |ignore: bool| {
            drawer.gesture.update_value(|gesture| {
                gesture.ignore_touch = ignore;
                gesture.touch = None;
            });
        };
        let from_pen = drawer
            .gesture
            .with_value(|gesture| gesture.pointer_type == "pen" && gesture.pen_touch);
        if from_pen {
            drawer
                .gesture
                .update_value(|gesture| gesture.pen_touch = false);
            stop(false);
            return;
        }
        if !untrack(|| self.open()) || !self.enabled() {
            stop(false);
            return;
        }
        let Some(touch) = event.touches().get(0) else {
            return;
        };
        if on_range(event) {
            stop(false);
            return;
        }
        let Some(root) = drawer.viewport.get_untracked() else {
            return;
        };
        let root: HtmlElement = root.into();
        let target = event
            .target()
            .and_then(|target| target.dyn_into::<Element>().ok())
            .unwrap_or_else(|| root.clone().into());
        if !root.contains(Some(&target)) {
            stop(true);
            return;
        }
        let at = (f64::from(touch.client_x()), f64::from(touch.client_y()));
        let ignored = element_at(at)
            .is_some_and(|element| element.closest(SWIPE_IGNORE).ok().flatten().is_some());
        if ignored {
            stop(true);
            return;
        }
        let direction = drawer.direction.get_untracked();
        let axis = axis_of(direction);
        let cross = match axis {
            Orientation::Horizontal => Orientation::Vertical,
            Orientation::Vertical => Orientation::Horizontal,
        };
        let scroll = scrollable_from(&target, &root, axis);
        let allow = match &scroll {
            Some(scroll) if !at_start_edge(scroll, axis, direction) => Some(false),
            _ => None,
        };
        let state = TouchScroll {
            start: at,
            last: at,
            cross: scrollable_from(&target, &root, cross).is_some(),
            target: scroll.map(SendWrapper::new),
            allow,
            preserve: false,
            attributed: false,
        };
        drawer.gesture.update_value(|gesture| {
            gesture.ignore_touch = false;
            gesture.touch = Some(state);
        });
        self.start(Input::Touch(event, &root));
    }

    fn touch_end(self, event: &web_sys::TouchEvent) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        drawer.gesture.update_value(|gesture| {
            gesture.ignore_touch = false;
            gesture.touch = None;
            gesture.pointer_type.clear();
            gesture.pen_touch = false;
        });
        let Some(root) = drawer.viewport.get_untracked() else {
            return;
        };
        let root: HtmlElement = root.into();
        self.end(Input::Touch(event, &root));
    }

    fn touch_move(self, event: &web_sys::TouchEvent) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        let Some(Some(mut state)) = drawer.gesture.try_with_value(|gesture| {
            (!gesture.ignore_touch)
                .then(|| gesture.touch.clone())
                .flatten()
        }) else {
            return;
        };
        let Some(touch) = event.touches().get(0) else {
            return;
        };
        let Some(root) = drawer.viewport.get_untracked() else {
            return;
        };
        let root: HtmlElement = root.into();
        let at = (f64::from(touch.client_x()), f64::from(touch.client_y()));
        self.claim_touch(event, &mut state, at, &root);
        state.last = at;
        drawer.gesture.update_value(|gesture| {
            if gesture.touch.is_some() {
                gesture.touch = Some(state);
            }
        });
    }

    fn claim_touch(
        self,
        event: &web_sys::TouchEvent,
        state: &mut TouchScroll,
        at: (f64, f64),
        root: &HtmlElement,
    ) {
        let Some(drawer) = self.try_drawer() else {
            return;
        };
        let direction = drawer.direction.get_untracked();
        let axis = axis_of(direction);
        let along_axis = |point: (f64, f64)| match axis {
            Orientation::Vertical => point.1,
            Orientation::Horizontal => point.0,
        };
        let delta = along_axis(at) - along_axis(state.last);
        // pinch zoom, or ios adjusting a selection
        if event.touches().length() == 2 {
            return;
        }
        if selecting(root) || !untrack(|| self.open()) || !self.enabled() {
            return;
        }
        if yields(state, event, at, axis) {
            return;
        }
        let claim = |event: &web_sys::TouchEvent| {
            if event.cancelable() {
                event.prevent_default();
            }
            event.stop_propagation();
        };
        let page = |target: &HtmlElement| {
            let document = document();
            document
                .body()
                .is_some_and(|body| body.is_same_node(Some(target)))
                || document
                    .document_element()
                    .is_some_and(|html| html.is_same_node(Some(target)))
        };
        let Some(target) = state
            .target
            .as_deref()
            .cloned()
            .filter(|target| !page(target))
        else {
            claim(event);
            self.moved(Input::Touch(event, root));
            return;
        };
        if scroll_room(&target, axis).1 <= 0.0 {
            claim(event);
            return;
        }
        if delta != 0.0 {
            let edge = can_leave_edge(&target, axis, direction, delta);
            if state.allow == Some(true) {
                if event.cancelable() {
                    event.prevent_default();
                }
            } else if event.cancelable() && edge {
                state.allow = Some(true);
                event.prevent_default();
            } else {
                state.allow = Some(false);
            }
        }
        if state.allow == Some(true) {
            event.stop_propagation();
            self.moved(Input::Touch(event, root));
        }
    }
}

fn yields(
    state: &mut TouchScroll,
    event: &web_sys::TouchEvent,
    at: (f64, f64),
    axis: Orientation,
) -> bool {
    if state.preserve {
        return true;
    }
    if state.attributed || state.allow == Some(true) || !state.cross {
        return false;
    }
    if !event.cancelable() {
        state.preserve = true;
        return true;
    }
    let (x, y) = (at.0 - state.start.0, at.1 - state.start.1);
    let (along, across) = match axis {
        Orientation::Vertical => (y.abs(), x.abs()),
        Orientation::Horizontal => (x.abs(), y.abs()),
    };
    if across >= AXIS_LOCK_SLOP && across > along + AXIS_LOCK_BIAS {
        state.preserve = true;
        return true;
    }
    if along >= AXIS_LOCK_SLOP {
        state.attributed = true;
        return false;
    }
    // ios cancels a scroll for the whole gesture once its first move is prevented
    true
}

fn scroll_room(element: &HtmlElement, axis: Orientation) -> (f64, f64) {
    match axis {
        Orientation::Vertical => (
            f64::from(element.scroll_top()),
            f64::from((element.scroll_height() - element.client_height()).max(0)),
        ),
        Orientation::Horizontal => (
            f64::from(element.scroll_left()),
            f64::from((element.scroll_width() - element.client_width()).max(0)),
        ),
    }
}

fn from_start(direction: SwipeDirection, axis: Orientation) -> bool {
    match axis {
        Orientation::Vertical => direction == SwipeDirection::Down,
        Orientation::Horizontal => direction == SwipeDirection::Right,
    }
}

fn at_start_edge(element: &HtmlElement, axis: Orientation, direction: SwipeDirection) -> bool {
    let (offset, max) = scroll_room(element, axis);
    if from_start(direction, axis) {
        offset <= 0.0
    } else {
        offset >= max
    }
}

fn can_leave_edge(
    element: &HtmlElement,
    axis: Orientation,
    direction: SwipeDirection,
    delta: f64,
) -> bool {
    let toward = if from_start(direction, axis) {
        delta > 0.0
    } else {
        delta < 0.0
    };
    toward && at_start_edge(element, axis, direction)
}

fn scrollable(element: &HtmlElement, axis: Orientation) -> bool {
    let Ok(Some(style)) = window().get_computed_style(element) else {
        return false;
    };
    let property = match axis {
        Orientation::Vertical => "overflow-y",
        Orientation::Horizontal => "overflow-x",
    };
    let overflow = style.get_property_value(property).unwrap_or_default();
    (overflow == "auto" || overflow == "scroll") && scroll_room(element, axis).1 > 0.0
}

fn last_traversable(element: &Element) -> bool {
    matches!(
        element.tag_name().to_ascii_lowercase().as_str(),
        "html" | "body"
    )
}

fn scrollable_from(target: &Element, root: &HtmlElement, axis: Orientation) -> Option<HtmlElement> {
    let mut node = Some(target.clone());
    while let Some(element) = node {
        if root.is_same_node(Some(&element)) || last_traversable(&element) {
            break;
        }
        if let Some(element) = element.dyn_ref::<HtmlElement>()
            && scrollable(element, axis)
        {
            return Some(element.clone());
        }
        node = element.parent_element();
    }
    scrollable(root, axis).then(|| root.clone())
}

fn gesture_scroll_target(
    target: &Element,
    root: &HtmlElement,
    horizontal: bool,
    vertical: bool,
) -> Option<HtmlElement> {
    let find = |axis| scrollable_from(target, root, axis).filter(|found| !last_traversable(found));
    match (horizontal, vertical) {
        (true, false) => find(Orientation::Horizontal),
        (false, true) => find(Orientation::Vertical),
        _ => find(Orientation::Vertical).or_else(|| find(Orientation::Horizontal)),
    }
}

fn scrollable_ancestor(
    target: &Element,
    root: &HtmlElement,
    horizontal: bool,
    vertical: bool,
) -> bool {
    let mut node = Some(target.clone());
    while let Some(element) = node {
        if root.is_same_node(Some(&element)) || last_traversable(&element) {
            return false;
        }
        if let Some(element) = element.dyn_ref::<HtmlElement>()
            && ((vertical && scrollable(element, Orientation::Vertical))
                || (horizontal && scrollable(element, Orientation::Horizontal)))
        {
            return true;
        }
        node = element.parent_element();
    }
    false
}

fn element_of(node: Option<web_sys::Node>) -> Option<Element> {
    let node = node?;
    match node.dyn_into::<Element>() {
        Ok(element) => Some(element),
        Err(node) => node.parent_element(),
    }
}

fn selecting(root: &HtmlElement) -> bool {
    let document = document();
    if let Some(active) = document.active_element()
        && root.contains(Some(&active))
    {
        let range = if let Some(input) = active.dyn_ref::<HtmlInputElement>() {
            (
                input.selection_start().ok().flatten(),
                input.selection_end().ok().flatten(),
            )
        } else if let Some(area) = active.dyn_ref::<HtmlTextAreaElement>() {
            (
                area.selection_start().ok().flatten(),
                area.selection_end().ok().flatten(),
            )
        } else {
            (None, None)
        };
        if let (Some(start), Some(end)) = range
            && start < end
        {
            return true;
        }
    }
    let Ok(Some(selection)) = document.get_selection() else {
        return false;
    };
    if selection.is_collapsed() {
        return false;
    }
    selection
        .contains_node_with_allow_partial_containment(root, true)
        .unwrap_or(false)
        || [selection.anchor_node(), selection.focus_node()]
            .into_iter()
            .filter_map(element_of)
            .any(|element| root.contains(Some(&element)))
}

fn clear_selection(popup: &HtmlElement) {
    let Ok(Some(selection)) = document().get_selection() else {
        return;
    };
    if selection.is_collapsed() {
        return;
    }
    let inside = [selection.anchor_node(), selection.focus_node()]
        .into_iter()
        .filter_map(element_of)
        .any(|element| popup.contains(Some(&element)));
    if inside {
        let _ = selection.remove_all_ranges();
    }
}

fn on_range(event: &web_sys::TouchEvent) -> bool {
    event.composed_path().iter().any(|target| {
        target
            .dyn_ref::<HtmlInputElement>()
            .is_some_and(|input| input.type_() == "range")
    })
}

#[leptos::component]
pub fn DrawerRoot(
    #[prop(optional, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, ChangeReason)>>,
    #[prop(into, default = true.into())] modal: Signal<bool>,
    #[prop(optional, into)] disable_pointer_dismissal: Signal<bool>,
    #[prop(into, default = SwipeDirection::Down.into())] swipe_direction: Signal<SwipeDirection>,
    #[prop(optional, into)] snap_points: Signal<Vec<DrawerSnapPoint>>,
    #[prop(optional, into)] snap_point: Option<Signal<Option<DrawerSnapPoint>>>,
    #[prop(optional)] default_snap_point: Option<DrawerSnapPoint>,
    #[prop(optional, into)] on_snap_point_change: Option<
        Callback<(Option<DrawerSnapPoint>, ChangeReason)>,
    >,
    #[prop(optional, into)] snap_to_sequential_points: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let parent = use_context::<DrawerContext>();
    let default_snap = Signal::derive(move || {
        default_snap_point.or_else(|| snap_points.with(|points| points.first().copied()))
    });
    let snap = Controlled::new(
        snap_point,
        default_snap.get_untracked(),
        on_snap_point_change,
    );
    // closing goes back to the first snap point
    let change = Callback::new(move |(open, reason): (bool, ChangeReason)| {
        if let Some(on_open_change) = on_open_change {
            on_open_change.run((open, reason));
        }
        if !open && snap_points.with_untracked(|points| !points.is_empty()) {
            snap.set(default_snap.get_untracked(), reason);
        }
    });
    let state = Controlled::new(open, default_open, Some(change));
    let dialog = DialogContext::new(
        state,
        modal,
        disable_pointer_dismissal,
        "dialog",
        InitialFocus::Popup,
    );
    let popup_height = RwSignal::new(0.0);
    let viewport_height = RwSignal::new(0.0);
    let rem = RwSignal::new(16.0);
    let resolved = Memo::new(move |_| {
        snap_points
            .with(|points| resolve(points, viewport_height.get(), popup_height.get(), rem.get()))
    });
    let context = DrawerContext(StoredValue::new(Drawer {
        dialog,
        parent,
        nested: RwSignal::new(Vec::new()),
        direction: swipe_direction,
        snap_points,
        snap,
        default_snap,
        sequential: snap_to_sequential_points,
        popup_height,
        viewport_height,
        rem,
        resolved,
        viewport: NodeRef::new(),
        backdrop: NodeRef::new(),
        swiping: RwSignal::new(false),
        swipe_active: RwSignal::new(false),
        progress: RwSignal::new(0.0),
        strength: RwSignal::new(None),
        gesture: StoredValue::new(Gesture::default()),
    }));
    if let Some(parent) = parent.and_then(DrawerContext::try_drawer) {
        parent.nested.update(|nested| nested.push(context));
        on_cleanup(move || {
            parent.nested.try_update(|nested| {
                nested.retain(|child| child.0 != context.0);
            });
        });
    }
    // tailwind's group variants match any ancestor, so inside its parent's popup it would take on the parent's state
    if parent.is_some() {
        Effect::new(move |_| {
            if let Some(element) = dialog.dialog.get()
                && let Some(body) = document().body()
            {
                let _ = body.append_child(&element);
            }
        });
    }
    view! {
        <Provider value=dialog>
            <Provider value=context>{children()}</Provider>
        </Provider>
    }
}

#[component(element = "div", skip(role))]
pub fn DrawerBackdrop(#[prop(optional)] force_render: bool) -> impl IntoView {
    let dialog = expect_context::<DialogContext>();
    let drawer = expect_context::<DrawerContext>().drawer();
    let open = move || dialog.open();
    // a nested drawer shows its parent's
    (force_render || drawer.parent.is_none()).then(move || {
        view! {
            <div
                {..attrs}
                {..transition_attributes(dialog.phase)}
                node_ref=drawer.backdrop
                role="presentation"
                style:pointer-events=move || (!open()).then_some("none")
                style:user-select="none"
                style:-webkit-user-select="none"
                style:--drawer-swipe-progress="0"
                style:--drawer-swipe-strength="1"
                data-open=move || open().then_some("")
                data-closed=move || (!open()).then_some("")
                on:click=move |_| {
                    if !dialog.disable_pointer_dismissal.get_untracked() {
                        dialog.native.state().set(false, ChangeReason::OutsidePress);
                    }
                }
            ></div>
        }
    })
}

#[component(element = "div", skip(role))]
pub fn DrawerViewport(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let dialog = expect_context::<DialogContext>();
    let context = expect_context::<DrawerContext>();
    let drawer = context.drawer();
    let open = move || dialog.open();

    Effect::new(move |_| {
        let Some(viewport) = drawer.viewport.get() else {
            return;
        };
        let measure = move || {
            if let Some(viewport) = drawer.viewport.get_untracked() {
                drawer
                    .viewport_height
                    .set(f64::from(viewport.offset_height()));
            }
            let size = document()
                .document_element()
                .and_then(|html| window().get_computed_style(&html).ok().flatten())
                .and_then(|style| style.get_property_value("font-size").ok())
                .and_then(|size| size.trim_end_matches("px").parse::<f64>().ok());
            if let Some(size) = size.filter(|size| size.is_finite()) {
                drawer.rem.set(size);
            }
        };
        measure();
        observe_resize(&viewport, measure);
    });

    // passive by default on the document, where it couldn't stop the page scrolling under a drag
    Effect::new(move |_| {
        let listener = leptos::wasm_bindgen::closure::Closure::<dyn Fn(web_sys::TouchEvent)>::new(
            move |event: web_sys::TouchEvent| context.touch_move(&event),
        );
        let options = web_sys::AddEventListenerOptions::new();
        options.set_capture(true);
        options.set_passive(false);
        let _ = document().add_event_listener_with_callback_and_add_event_listener_options(
            "touchmove",
            listener.as_ref().unchecked_ref(),
            &options,
        );
        let held = StoredValue::new_local(listener);
        on_cleanup(move || {
            held.try_with_value(|listener| {
                let _ = document().remove_event_listener_with_callback_and_bool(
                    "touchmove",
                    listener.as_ref().unchecked_ref(),
                    true,
                );
            });
        });
    });

    Effect::new(move |_| {
        if dialog.open() {
            context.reset();
            context.clear_release();
        } else if drawer.parent.is_some() {
            drawer.progress.set(0.0);
        }
    });

    Effect::new(move |_| {
        if context.snap_range().is_none() || drawer.swiping.get() {
            return;
        }
        let progress = if dialog.open() && drawer.parent.is_none() {
            context.snap_progress().unwrap_or(0.0)
        } else {
            0.0
        };
        context.frontmost();
        context.apply_progress(progress, false);
    });

    let at_point = |event: &ev::PointerEvent| {
        element_at((f64::from(event.client_x()), f64::from(event.client_y())))
    };
    let pointer_end = move |event: &ev::PointerEvent| {
        drawer
            .gesture
            .update_value(|gesture| gesture.pointer_type.clear());
        event.pointer_type() != "touch"
    };

    view! {
        <div
            {..attrs}
            {..transition_attributes(dialog.phase)}
            node_ref=drawer.viewport
            role="presentation"
            style:pointer-events=move || (!open()).then_some("none")
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-nested=drawer.parent.is_some().then_some("")
            on:pointerdown=move |event: ev::PointerEvent| {
                drawer
                    .gesture
                    .update_value(|gesture| {
                        gesture.pointer_type = event.pointer_type();
                        gesture.pen_touch = event.pointer_type() == "pen";
                    });
                if !untrack(open) || !context.enabled() {
                    return;
                }
                let skipped = at_point(&event)
                    .is_some_and(|element| {
                        element.closest(SWIPE_IGNORE).ok().flatten().is_some()
                            || element.closest(CONTENT).ok().flatten().is_some()
                    });
                if skipped || event.pointer_type() == "touch" {
                    return;
                }
                context.start(Input::Pointer(&event));
            }
            on:pointermove=move |event: ev::PointerEvent| {
                if event.pointer_type() != "touch" {
                    context.moved(Input::Pointer(&event));
                }
            }
            on:pointerup=move |event: ev::PointerEvent| {
                if pointer_end(&event) {
                    context.end(Input::Pointer(&event));
                }
            }
            on:pointercancel=move |event: ev::PointerEvent| {
                if pointer_end(&event) {
                    context.end(Input::Pointer(&event));
                }
            }
            on:touchstart=move |event: ev::TouchEvent| context.touch_start(&event)
            on:touchend=move |event: ev::TouchEvent| context.touch_end(&event)
            on:touchcancel=move |event: ev::TouchEvent| context.touch_end(&event)
            on:click=move |event: ev::MouseEvent| {
                let outside = event
                    .target()
                    .and_then(|target| target.dyn_into::<web_sys::Node>().ok())
                    .is_some_and(|target| {
                        !dialog
                            .popup
                            .get_untracked()
                            .is_some_and(|popup| popup.contains(Some(&target)))
                    });
                if outside && dialog.modal.get_untracked()
                    && !dialog.disable_pointer_dismissal.get_untracked()
                {
                    dialog.native.state().set(false, ChangeReason::OutsidePress);
                }
            }
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div", skip(tabindex))]
pub fn DrawerPopup(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let dialog = expect_context::<DialogContext>();
    let context = expect_context::<DrawerContext>();
    let drawer = context.drawer();
    let open = move || dialog.open();

    let measure = move || {
        let Some(popup) = dialog.popup.get_untracked() else {
            return;
        };
        let height = f64::from(popup.offset_height());
        let previous = drawer.popup_height.get_untracked();
        let (frontmost, nested) = untrack(|| (context.frontmost(), context.has_nested()));
        if previous > 0.0 && ((frontmost > previous && height > previous) || nested) {
            return;
        }
        if height != previous {
            drawer.popup_height.set(height);
        }
    };
    Effect::new(move |_| {
        context.nested_open();
        if !context.present() {
            drawer.popup_height.set(0.0);
            return;
        }
        if let Some(popup) = dialog.popup.get() {
            measure();
            observe_resize(&popup, measure);
        }
    });

    let height = move || {
        let height = drawer.popup_height.get();
        let fixed = context.has_nested() || dialog.phase.get() == TransitionStatus::Ending;
        (fixed && height > 0.0).then(|| format!("{height}px"))
    };
    let snap_offset = move || {
        let direction = drawer.direction.get();
        let offset = (context.has_snap_points() && vertical(direction))
            .then(|| context.active_offset())
            .flatten()
            .map_or(0.0, |offset| {
                if direction == SwipeDirection::Up {
                    -offset
                } else {
                    offset
                }
            });
        format!("{offset}px")
    };
    let strength = move || {
        drawer
            .strength
            .get()
            .filter(|strength| strength.is_finite() && *strength > 0.0)
            .map_or_else(|| "1".to_string(), |strength| strength.to_string())
    };

    view! {
        <div
            {..attrs}
            {..transition_attributes(dialog.phase)}
            node_ref=dialog.popup
            tabindex="-1"
            style:pointer-events="auto"
            style:--drawer-swipe-movement-x="0px"
            style:--drawer-swipe-movement-y="0px"
            style:--drawer-swipe-progress=move || context.nested_progress().to_string()
            style:--nested-drawers=move || context.nested_open().to_string()
            style:--drawer-height=height
            style:--drawer-snap-point-offset=snap_offset
            style:--drawer-frontmost-height=move || {
                let frontmost = context.frontmost();
                (frontmost > 0.0).then(|| format!("{frontmost}px"))
            }
            style:--drawer-swipe-strength=strength
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-nested=drawer.parent.is_some().then_some("")
            data-expanded=move || context.expanded().then_some("")
            data-nested-drawer-open=move || (context.nested_open() > 0).then_some("")
            data-nested-drawer-swiping=move || context.nested_swiping().then_some("")
            data-swipe-direction=move || drawer.direction.get().as_str()
            data-swiping=move || drawer.swiping.get().then_some("")
            on:keydown=move |event: ev::KeyboardEvent| {
                if matches!(
                    event.key().as_str(),
                    "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" | "Home" | "End"
                ) {
                    event.stop_propagation();
                }
            }
        >
            {children.map(|children| children())}
        </div>
    }
}

// a press here doesn't start a mouse swipe, a touch still does
#[component(element = "div")]
pub fn DrawerContent(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <div {..attrs} data-drawer-content="">
            {children.map(|children| children())}
        </div>
    }
}
