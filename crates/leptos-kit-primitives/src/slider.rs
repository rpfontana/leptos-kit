use crate::id::ElementId;
use crate::native::{along, change, client, observe_resize};
use crate::{
    ChangeReason, Controlled, Direction, Keyword, Orientation, keyword, use_direction, use_id,
};
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_kit_macros::component;
use web_sys::{FocusOptions, HtmlElement, HtmlInputElement, MouseEvent, Node};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Keyword)]
pub enum ThumbAlignment {
    Center,
    Edge,
    EdgeClientOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Keyword)]
pub enum ThumbCollisionBehavior {
    Push,
    Swap,
    None,
}

#[derive(Clone, Copy)]
struct Thumb {
    key: usize,
    element: NodeRef<html::Div>,
    input: NodeRef<html::Input>,
    input_id: ElementId<html::Input>,
    disabled: Signal<bool>,
    // its length along the axis, which an edge-aligned indicator needs
    size: RwSignal<f64>,
}

struct Finger {
    values: Vec<f64>,
    thumb: usize,
    swapped: bool,
}

#[derive(Clone, Copy)]
struct Bounds {
    min: f64,
    max: f64,
    gap: f64,
}

impl Bounds {
    #[expect(
        clippy::as_conversions,
        clippy::cast_precision_loss,
        reason = "a slider has a handful of thumbs"
    )]
    fn gaps(self, count: usize) -> f64 {
        count as f64 * self.gap
    }
}

// a press on the control, from pointerdown to pointerup
#[derive(Clone, Default)]
struct Press {
    pointer: Option<i32>,
    thumb: Option<usize>,
    // how far from its centre the pressed thumb was grabbed
    offset: Option<f64>,
    inset: f64,
    initial: Option<Vec<f64>>,
    latest: Vec<f64>,
    changed: Option<Vec<f64>>,
    reason: ChangeReason,
    moves: u32,
}

#[derive(Clone, Copy)]
struct SliderContext {
    value: Controlled<Vec<f64>>,
    values: Memo<Vec<f64>>,
    on_value_committed: Option<Callback<(Vec<f64>, ChangeReason)>>,
    min: Signal<f64>,
    max: Signal<f64>,
    step: Signal<f64>,
    large_step: Signal<f64>,
    min_steps: Signal<f64>,
    disabled: Signal<bool>,
    orientation: Signal<Orientation>,
    edge: Signal<bool>,
    collision: Signal<ThumbCollisionBehavior>,
    name: Option<Signal<String>>,
    form: Option<Signal<String>>,
    format: Option<Callback<f64, String>>,
    direction: Signal<Direction>,
    active: RwSignal<Option<usize>>,
    last_used: RwSignal<Option<usize>>,
    dragging: RwSignal<bool>,
    control: NodeRef<html::Div>,
    thumbs: RwSignal<Vec<Thumb>>,
    label: RwSignal<Option<ElementId<html::Div>>>,
    press: StoredValue<Press>,
    // the thumb a press focuses on the next frame, which a swap before then moves
    focus: StoredValue<Option<usize>>,
    next: StoredValue<usize>,
}

impl SliderContext {
    fn axis(self) -> Orientation {
        self.orientation.get_untracked()
    }

    fn formatted(self, value: f64) -> String {
        self.format
            .map_or_else(|| format_number(value), |format| format.run(value))
    }

    fn percent(self, value: f64) -> f64 {
        let (min, max) = (self.min.get(), self.max.get());
        (value - min) * 100.0 / (max - min)
    }

    fn set_active(self, index: Option<usize>) {
        change(self.active, index);
        if index.is_some() {
            change(self.last_used, index);
        }
    }

    // true when it asked for a new value
    fn set(self, next: Vec<f64>, reason: ChangeReason) -> bool {
        if next.iter().any(|value| value.is_nan()) || self.value.with_untracked(|now| *now == next)
        {
            return false;
        }
        self.value.set(next, reason);
        true
    }

    fn commit(self, value: Vec<f64>, reason: ChangeReason) {
        if let Some(on_value_committed) = self.on_value_committed {
            on_value_committed.run((value, reason));
        }
    }

    // a key or the input itself moved one thumb, which stays between its neighbours
    fn input_change(self, input: f64, index: usize, reason: ChangeReason) {
        let (min, max) = (self.min.get_untracked(), self.max.get_untracked());
        let values = self.values.get_untracked();
        let mut next = values.clone();
        if let Some(value) = next.get_mut(index) {
            let low = index
                .checked_sub(1)
                .map_or(f64::NEG_INFINITY, |i| values[i]);
            let high = values.get(index + 1).copied().unwrap_or(f64::INFINITY);
            *value = clamp(clamp(input, min, max), low, high);
        }
        next.sort_by(f64::total_cmp);
        let gap = self.step.get_untracked() * self.min_steps.get_untracked();
        if spaced(&next, gap) && self.set(next.clone(), reason) {
            self.commit(next, reason);
        }
    }

    fn focus_thumb(self, index: usize) {
        let input = self
            .thumbs
            .with_untracked(|thumbs| thumbs.get(index).map(|thumb| thumb.input))
            .and_then(|input| input.get_untracked());
        if let Some(input) = input {
            let options = FocusOptions::new();
            options.set_prevent_scroll(true);
            let _ = input.focus_with_options(&options);
        }
    }

    fn start_press(self, x: f64, y: f64) {
        let values = self.values.get_untracked();
        let axis = self.axis();
        let max = self.max.get_untracked();
        let pressed = self
            .press
            .with_value(|press| press.thumb)
            .filter(|index| *index < values.len());
        let closest = match pressed {
            // thumbs stacked at the max: the first of them can still move down
            Some(mut index) => {
                while index > 0 && values[index] == max && values[index - 1] == max {
                    index -= 1;
                }
                Some(index)
            }
            None => self.thumbs.with_untracked(|thumbs| {
                let point = match axis {
                    Orientation::Horizontal => x,
                    Orientation::Vertical => y,
                };
                let mut closest: Option<(usize, f64)> = None;
                for (index, thumb) in thumbs.iter().enumerate() {
                    let Some(element) = thumb.element.get_untracked() else {
                        continue;
                    };
                    if thumb.disabled.get_untracked() {
                        continue;
                    }
                    let distance = (point - midpoint(&element, axis)).abs();
                    if closest.is_none_or(|(_, best)| distance <= best) {
                        closest = Some((index, distance));
                    }
                }
                closest.map(|(index, _)| index)
            }),
        };
        let inset = match (self.edge.get_untracked(), closest) {
            (true, Some(index)) => self.thumbs.with_untracked(|thumbs| {
                thumbs
                    .get(index)
                    .map_or(0.0, |thumb| thumb.size.get_untracked() / 2.0)
            }),
            _ => 0.0,
        };
        self.press.update_value(|press| {
            press.initial = (values.len() > 1).then(|| values.clone());
            press.latest = values;
            press.changed = None;
            press.thumb = closest;
            press.inset = inset;
        });
    }

    // where a finger at x, y puts the pressed thumb, and which thumb that is after a swap
    fn finger(self, x: f64, y: f64) -> Option<Finger> {
        let control = self.control.get_untracked()?;
        let values = self.values.get_untracked();
        let press = self.press.get_value();
        let index = press.thumb.filter(|index| *index < values.len())?;
        let axis = self.axis();
        let rect = control.get_bounding_client_rect();
        let (start, end) = edges(&control, axis);
        let length = match axis {
            Orientation::Horizontal => rect.width(),
            Orientation::Vertical => rect.height(),
        } - start
            - end
            - press.inset * 2.0;
        let offset = press.offset.unwrap_or(0.0);
        let along = match axis {
            Orientation::Vertical => rect.bottom() - (y - offset) - end,
            Orientation::Horizontal if self.direction.get_untracked() == Direction::Rtl => {
                rect.right() - (x - offset) - start
            }
            Orientation::Horizontal => x - offset - rect.left() - start,
        };
        let ratio = clamp((along - press.inset) / length, 0.0, 1.0);
        let (min, max, step) = (
            self.min.get_untracked(),
            self.max.get_untracked(),
            self.step.get_untracked(),
        );
        let value = clamp(
            round_to_step((max - min) * ratio + min, step, min),
            min,
            max,
        );
        if values.len() < 2 {
            return Some(Finger {
                values: vec![value],
                thumb: 0,
                swapped: false,
            });
        }
        let bounds = Bounds {
            min,
            max,
            gap: step * self.min_steps.get_untracked(),
        };
        Some(resolve_collision(
            self.collision.get_untracked(),
            &press.latest,
            press.initial.as_deref().unwrap_or(&values),
            index,
            value,
            bounds,
        ))
    }

    fn set_from_pointer(self, finger: Finger, reason: ChangeReason) {
        let Finger {
            values,
            thumb,
            swapped,
        } = finger;
        if !self.set(values.clone(), reason) {
            return;
        }
        self.press.update_value(|press| {
            press.changed = Some(values.clone());
            press.latest = values;
            press.reason = reason;
            if swapped {
                press.thumb = Some(thumb);
            }
        });
        if swapped {
            self.focus
                .update_value(|focus| *focus = focus.map(|_| thumb));
            self.focus_thumb(thumb);
        }
    }

    fn release(self, event: &ev::PointerEvent) {
        let press = self.press.get_value();
        if press.pointer != Some(event.pointer_id()) {
            return;
        }
        change(self.active, None);
        change(self.dragging, false);
        self.press.set_value(Press::default());
        let count = self.values.with_untracked(Vec::len);
        if let Some(changed) = press.changed.filter(|changed| changed.len() == count) {
            self.commit(changed, press.reason);
        }
        if let Some(control) = self.control.get_untracked()
            && control.has_pointer_capture(event.pointer_id())
        {
            let _ = control.release_pointer_capture(event.pointer_id());
        }
    }
}

// base ui's clamp: never panics, the upper bound wins
fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

fn fixed(value: f64, digits: usize) -> f64 {
    format!("{value:.digits$}").parse().unwrap_or(value)
}

fn precision(value: f64) -> usize {
    let text = value.abs().to_string();
    text.split_once('.')
        .map_or(0, |(_, fraction)| fraction.len())
}

fn round_to_step(value: f64, step: f64, min: f64) -> f64 {
    let nearest = ((value - min) / step).round() * step + min;
    fixed(nearest, precision(step).max(precision(min)))
}

fn spaced(values: &[f64], gap: f64) -> bool {
    values
        .windows(2)
        .all(|pair| (pair[0] - pair[1]).abs() >= gap)
}

fn midpoint(element: &HtmlElement, axis: Orientation) -> f64 {
    let rect = element.get_bounding_client_rect();
    match axis {
        Orientation::Horizontal => f64::midpoint(rect.left(), rect.right()),
        Orientation::Vertical => f64::midpoint(rect.top(), rect.bottom()),
    }
}

// border and padding at the control's start and end
fn edges(element: &HtmlElement, axis: Orientation) -> (f64, f64) {
    let Some(style) = window().get_computed_style(element).ok().flatten() else {
        return (0.0, 0.0);
    };
    let read = |name: &str| {
        style
            .get_property_value(name)
            .ok()
            .and_then(|value| value.trim_end_matches("px").parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    let side = match axis {
        Orientation::Horizontal => ["inline-start", "inline-end"],
        Orientation::Vertical => ["top", "bottom"],
    };
    let [start, end] =
        side.map(|side| read(&format!("border-{side}-width")) + read(&format!("padding-{side}")));
    (start, end)
}

// base ui formats with the page's intl locale, english here so the server agrees
fn format_number(value: f64) -> String {
    let text = fixed(value, 3).to_string();
    let (sign, digits) = text
        .strip_prefix('-')
        .map_or(("", text.as_str()), |rest| ("-", rest));
    let (whole, fraction) = digits
        .split_once('.')
        .map_or((digits, None), |(whole, fraction)| (whole, Some(fraction)));
    let mut grouped = String::new();
    for (index, digit) in whole.chars().enumerate() {
        if index > 0 && (whole.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    match fraction {
        Some(fraction) => format!("{sign}{grouped}.{fraction}"),
        None => format!("{sign}{grouped}"),
    }
}

// base ui's getPushedThumbValues: the moved thumb pushes its neighbours, which spring back when it leaves
fn pushed(values: &[f64], index: usize, next: f64, bounds: Bounds, initial: &[f64]) -> Vec<f64> {
    let Bounds { min, max, gap } = bounds;
    let mut out = values.to_vec();
    let last = out.len() - 1;
    let base = |i: usize| initial.get(i).copied().unwrap_or(values[i]);
    out[index] = clamp(
        next,
        min + bounds.gaps(index),
        max - bounds.gaps(last - index),
    );
    for i in index + 1..=last {
        let (low, high) = (out[i - 1] + gap, max - bounds.gaps(last - i));
        let mut candidate = out[i].max(low);
        if base(i) < candidate {
            candidate = base(i).max(low);
        }
        out[i] = clamp(candidate, low, high);
    }
    for i in (0..index).rev() {
        let (low, high) = (min + bounds.gaps(i), out[i + 1] - gap);
        let mut candidate = out[i].min(high);
        if base(i) > candidate {
            candidate = base(i).min(high);
        }
        out[i] = clamp(candidate, low, high);
    }
    out.into_iter().map(|value| fixed(value, 12)).collect()
}

// base ui's resolveThumbCollision for push, swap and none
fn resolve_collision(
    behavior: ThumbCollisionBehavior,
    values: &[f64],
    initial: &[f64],
    pressed: usize,
    next: f64,
    bounds: Bounds,
) -> Finger {
    if behavior == ThumbCollisionBehavior::Push {
        return Finger {
            values: pushed(values, pressed, next, bounds, values),
            thumb: pressed,
            swapped: false,
        };
    }
    let Bounds { min, max, gap } = bounds;
    let mut candidate = values.to_vec();
    let previous = pressed.checked_sub(1).map(|i| candidate[i]);
    let following = candidate.get(pressed + 1).copied();
    let low = previous.map_or(min, |previous| previous + gap);
    let high = following.map_or(max, |following| following - gap);
    let clamped = fixed(clamp(next, low, high), 12);
    candidate[pressed] = clamped;
    let forward =
        next > values[pressed] && following.is_some_and(|following| next >= following - 1e-7);
    let backward =
        next < values[pressed] && previous.is_some_and(|previous| next <= previous + 1e-7);
    if behavior != ThumbCollisionBehavior::Swap || (!forward && !backward) {
        return Finger {
            values: candidate,
            thumb: pressed,
            swapped: false,
        };
    }
    let target = if forward { pressed + 1 } else { pressed - 1 };
    let baseline: Vec<f64> = (0..candidate.len())
        .map(|i| {
            if i == pressed {
                clamped
            } else {
                initial.get(i).copied().unwrap_or(values[i])
            }
        })
        .collect();
    let target_next = if forward {
        next.max(candidate[target])
    } else {
        next.min(candidate[target])
    };
    let mut adjusted = pushed(&candidate, target, target_next, bounds, &baseline);
    let neighbour = if forward { target - 1 } else { target + 1 };
    let low = neighbour
        .checked_sub(1)
        .map_or(min, |i| adjusted[i] + gap)
        .max(min + bounds.gaps(neighbour));
    let high = adjusted
        .get(neighbour + 1)
        .map_or(max, |following| following - gap)
        .min(max - bounds.gaps(adjusted.len() - 1 - neighbour));
    adjusted[neighbour] = fixed(clamp(clamped, low, high), 12);
    Finger {
        values: adjusted,
        thumb: target,
        swapped: true,
    }
}

fn state_attributes(slider: SliderContext) -> impl leptos::attr::Attribute {
    view! {
        <{..}
            data-disabled=move || slider.disabled.get().then_some("")
            data-dragging=move || slider.dragging.get().then_some("")
            data-orientation=keyword(slider.orientation)
        />
    }
}

#[component(element = "div", skip(role))]
pub fn SliderRoot(
    #[prop(optional_no_strip, into)] value: Option<Signal<Vec<f64>>>,
    #[prop(optional)] default_value: Vec<f64>,
    #[prop(optional_no_strip, into)] on_value_change: Option<Callback<(Vec<f64>, ChangeReason)>>,
    #[prop(optional_no_strip, into)] on_value_committed: Option<Callback<(Vec<f64>, ChangeReason)>>,
    #[prop(into, default = 0.0.into())] min: Signal<f64>,
    #[prop(into, default = 100.0.into())] max: Signal<f64>,
    #[prop(into, default = 1.0.into())] step: Signal<f64>,
    #[prop(into, default = 10.0.into())] large_step: Signal<f64>,
    #[prop(optional, into)] min_steps_between_values: Signal<f64>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(into, default = Orientation::Horizontal.into())] orientation: Signal<Orientation>,
    #[prop(into, default = ThumbAlignment::Center.into())] thumb_alignment: Signal<ThumbAlignment>,
    #[prop(into, default = ThumbCollisionBehavior::Push.into())] thumb_collision_behavior: Signal<
        ThumbCollisionBehavior,
    >,
    #[prop(optional_no_strip, into)] name: Option<Signal<String>>,
    #[prop(optional_no_strip, into)] form: Option<Signal<String>>,
    #[prop(optional_no_strip, into)] format: Option<Callback<f64, String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let default_value = if default_value.is_empty() {
        vec![min.get_untracked()]
    } else {
        default_value
    };
    let value = Controlled::new(value, default_value, on_value_change);
    let values = Memo::new(move |_| {
        let (min, max) = (min.get(), max.get());
        let mut values: Vec<f64> = value
            .get()
            .into_iter()
            .map(|value| clamp(value, min, max))
            .collect();
        values.sort_by(f64::total_cmp);
        values
    });
    let slider = SliderContext {
        value,
        values,
        on_value_committed,
        min,
        max,
        step,
        large_step,
        min_steps: min_steps_between_values,
        disabled,
        orientation,
        edge: Signal::derive(move || thumb_alignment.get() != ThumbAlignment::Center),
        collision: thumb_collision_behavior,
        name,
        form,
        format,
        direction: use_direction(),
        active: RwSignal::new(None),
        last_used: RwSignal::new(None),
        dragging: RwSignal::new(false),
        control: NodeRef::new(),
        thumbs: RwSignal::new(Vec::new()),
        label: RwSignal::new(None),
        press: StoredValue::new(Press::default()),
        focus: StoredValue::new(None),
        next: StoredValue::new(0),
    };
    let root = NodeRef::<html::Div>::new();
    // chrome leaves the focus on an input once it's disabled
    Effect::new(move || {
        if !disabled.get() {
            return;
        }
        let focused = document().active_element();
        if let (Some(root), Some(focused)) = (root.get_untracked(), focused)
            && root.contains(Some(&focused))
            && let Ok(focused) = focused.dyn_into::<HtmlElement>()
        {
            let _ = focused.blur();
        }
        change(slider.active, None);
    });
    view! {
        <div
            {..attrs}
            {..state_attributes(slider)}
            node_ref=root
            role="group"
            aria-labelledby=move || slider.label.get().map(|label| label.get())
        >
            <Provider value=slider>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "div", skip(id))]
pub fn SliderLabel(
    #[prop(optional_no_strip, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let slider = expect_context::<SliderContext>();
    let element = NodeRef::new();
    let label = ElementId::new(element, id.unwrap_or_else(use_id));
    slider.label.set(Some(label));
    on_cleanup(move || {
        slider.label.try_set(None);
    });
    // a click focuses the only thumb, there's no <label for> to do it
    let focus = move |_| {
        if slider.thumbs.with_untracked(Vec::len) == 1 {
            slider.focus_thumb(0);
        }
    };
    view! {
        <div {..attrs} {..state_attributes(slider)} node_ref=element id=label.get() on:click=focus>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "output", skip(r#for))]
pub fn SliderValue() -> impl IntoView {
    let slider = expect_context::<SliderContext>();
    let inputs = move || {
        slider.thumbs.with(|thumbs| {
            let ids: Vec<String> = thumbs.iter().map(|thumb| thumb.input_id.get()).collect();
            (!ids.is_empty()).then(|| ids.join(" "))
        })
    };
    let text = move || {
        let values = slider.values.get();
        values
            .into_iter()
            .map(|value| slider.formatted(value))
            .collect::<Vec<_>>()
            .join(" – ")
    };
    view! {
        <output {..attrs} {..state_attributes(slider)} aria-live="off" for=inputs>
            {text}
        </output>
    }
}

#[component(element = "div")]
pub fn SliderControl(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let slider = expect_context::<SliderContext>();
    let point = |event: &MouseEvent| (f64::from(event.client_x()), f64::from(event.client_y()));
    let press = move |event: ev::PointerEvent| {
        let Some(control) = slider.control.get_untracked() else {
            return;
        };
        if slider.disabled.get_untracked() || event.default_prevented() || event.button() != 0 {
            return;
        }
        let target = event
            .target()
            .and_then(|target| target.dyn_into::<Node>().ok());
        let on_disabled_thumb = slider.thumbs.with_untracked(|thumbs| {
            thumbs.iter().any(|thumb| {
                thumb.disabled.get_untracked()
                    && thumb
                        .element
                        .get_untracked()
                        .is_some_and(|element| element.contains(target.as_ref()))
            })
        });
        if on_disabled_thumb {
            slider.press.set_value(Press::default());
            return;
        }
        let (x, y) = point(&event);
        slider.start_press(x, y);
        let Some(finger) = slider.finger(x, y) else {
            return;
        };
        let thumb = slider.thumbs.with_untracked(|thumbs| {
            thumbs
                .get(finger.thumb)
                .and_then(|thumb| thumb.element.get_untracked())
        });
        let focused = document().active_element();
        if thumb.is_some_and(|thumb| thumb.contains(focused.as_ref().map(AsRef::as_ref))) {
            event.prevent_default();
        } else {
            slider.focus.set_value(Some(finger.thumb));
            request_animation_frame(move || {
                if let Some(Some(index)) = slider.focus.try_update_value(Option::take) {
                    slider.focus_thumb(index);
                }
            });
        }
        change(slider.dragging, true);
        let on_thumb = slider.press.with_value(|press| press.offset.is_some());
        slider.press.update_value(|press| {
            press.pointer = Some(event.pointer_id());
            press.moves = 0;
        });
        if !on_thumb {
            slider.set_from_pointer(finger, ChangeReason::TrackPress);
        }
        let _ = control.set_pointer_capture(event.pointer_id());
    };
    let drag = move |event: ev::PointerEvent| {
        let Press { pointer, moves, .. } = slider.press.get_value();
        if pointer != Some(event.pointer_id()) {
            return;
        }
        slider.press.update_value(|press| press.moves += 1);
        if event.buttons() == 0 {
            slider.release(&event);
            return;
        }
        let (x, y) = point(&event);
        let Some(finger) = slider.finger(x, y) else {
            return;
        };
        let gap = slider.step.get_untracked() * slider.min_steps.get_untracked();
        if spaced(&finger.values, gap) {
            // a couple of moves before it counts as a drag
            if moves + 1 > 2 {
                change(slider.dragging, true);
            }
            slider.set_from_pointer(finger, ChangeReason::Drag);
        }
    };
    view! {
        <div
            {..attrs}
            {..state_attributes(slider)}
            node_ref=slider.control
            on:pointerdown=press
            on:pointermove=drag
            on:pointerup=move |event: ev::PointerEvent| slider.release(&event)
            on:pointercancel=move |event: ev::PointerEvent| slider.release(&event)
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn SliderTrack(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let slider = expect_context::<SliderContext>();
    view! {
        <div {..attrs} {..state_attributes(slider)} style:position="relative">
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn SliderIndicator(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let slider = expect_context::<SliderContext>();
    let vertical = move || slider.orientation.get() == Orientation::Vertical;
    // a thumb's centre along the control, edge-aligned thumbs stay a half thumb inside
    let centre = move |index: usize| {
        let value = slider.values.with(|values| values.get(index).copied())?;
        let percent = slider.percent(value);
        let size = slider
            .thumbs
            .with(|thumbs| thumbs.get(index).map(|thumb| thumb.size.get()))
            .unwrap_or(0.0);
        let inside = if slider.edge.get() {
            (0.5 - percent / 100.0) * size
        } else {
            0.0
        };
        Some((percent, inside))
    };
    let start = move || centre(0).unwrap_or((0.0, 0.0));
    let end = move || {
        let last = slider.values.with(|values| values.len().saturating_sub(1));
        centre(last).unwrap_or((0.0, 0.0))
    };
    let range = move || slider.values.with(|values| values.len() > 1);
    let css = |(percent, px): (f64, f64)| format!("calc({percent}% + {px}px)");
    let offset = move || {
        if range() {
            css(start())
        } else {
            "0".to_string()
        }
    };
    let length = move || {
        if range() {
            let ((from, from_px), (to, to_px)) = (start(), end());
            css((to - from, to_px - from_px))
        } else {
            css(start())
        }
    };
    view! {
        <div
            {..attrs}
            {..state_attributes(slider)}
            style:position=move || if vertical() { "absolute" } else { "relative" }
            style:height=move || if vertical() { length() } else { "inherit".to_string() }
            style:width=move || if vertical() { "inherit".to_string() } else { length() }
            style:bottom=move || vertical().then(offset)
            style:inset-inline-start=move || (!vertical()).then(offset)
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div", skip(tabindex))]
pub fn SliderThumb(
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional_no_strip, into)] aria_label: Option<Signal<String>>,
    #[prop(optional_no_strip, into)] aria_labelledby: Option<Signal<String>>,
    #[prop(optional_no_strip, into)] aria_describedby: Option<Signal<String>>,
    #[prop(optional_no_strip, into)] aria_valuetext: Option<Signal<String>>,
    #[prop(optional_no_strip, into)] get_aria_label: Option<Callback<usize, String>>,
    #[prop(optional_no_strip, into)] get_aria_value_text: Option<
        Callback<(String, f64, usize), String>,
    >,
    #[prop(optional_no_strip, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let slider = expect_context::<SliderContext>();
    let key = slider
        .next
        .try_update_value(|next| {
            *next += 1;
            *next - 1
        })
        .unwrap_or_default();
    let element = NodeRef::<html::Div>::new();
    let input = NodeRef::<html::Input>::new();
    let disabled = Signal::derive(move || disabled.get() || slider.disabled.get());
    let thumb = Thumb {
        key,
        element,
        input,
        input_id: ElementId::new(input, use_id()),
        disabled,
        size: RwSignal::new(0.0),
    };
    slider.thumbs.update(|thumbs| thumbs.push(thumb));
    on_cleanup(move || {
        slider
            .thumbs
            .try_update(|thumbs| thumbs.retain(|thumb| thumb.key != key));
    });
    Effect::new(move || {
        let Some(element) = element.get() else {
            return;
        };
        let element: HtmlElement = element.into();
        let measure = {
            let element = element.clone();
            move || change(thumb.size, along(&element, slider.axis()))
        };
        measure();
        observe_resize(&element, measure);
    });

    let index = Memo::new(move |_| {
        slider
            .thumbs
            .with(|thumbs| thumbs.iter().position(|thumb| thumb.key == key))
            .unwrap_or_default()
    });
    let value = move || {
        slider
            .values
            .with(|values| values.get(index.get()).copied())
    };
    let vertical = move || slider.orientation.get() == Orientation::Vertical;
    let percent = move || value().map(|value| slider.percent(value));
    let start = move || percent().map(|percent| format!("{percent}%"));
    // edge alignment shifts the thumb by its own share, so it ends flush with the track
    let translate = move || {
        let percent = percent().unwrap_or(0.0);
        let shift = if slider.edge.get() { percent } else { 50.0 };
        if vertical() {
            format!("-50% {shift}%")
        } else if slider.direction.get() == Direction::Rtl {
            format!("{shift}% -50%")
        } else {
            format!("-{shift}% -50%")
        }
    };
    let z_index = move || {
        let (index, range) = (
            Some(index.get()),
            slider.values.with(|values| values.len() > 1),
        );
        if slider.active.get() == index {
            Some(if range { "2" } else { "1" })
        } else {
            (range && slider.last_used.get() == index).then_some("1")
        }
    };
    let label = move || {
        get_aria_label
            .map(|label| label.run(index.get()))
            .or_else(|| aria_label.map(|label| label.get()))
    };
    let labelled_by = move || match aria_labelledby {
        Some(labelled_by) => Some(labelled_by.get()),
        None if label().is_none() => slider.label.get().map(|label| label.get()),
        None => None,
    };
    let value_text = move || {
        let value = value()?;
        if let Some(text) = get_aria_value_text {
            return Some(text.run((slider.formatted(value), value, index.get())));
        }
        if let Some(text) = aria_valuetext {
            return Some(text.get());
        }
        if slider.values.with(Vec::len) == 2 {
            let end = if index.get() == 0 { "start" } else { "end" };
            return Some(format!("{} {end} range", slider.formatted(value)));
        }
        slider.format.map(|format| format.run(value))
    };
    let current = move || value().map(|value| value.to_string()).unwrap_or_default();
    let keydown = move |event: ev::KeyboardEvent| {
        if event.default_prevented() {
            return;
        }
        let index = index.get_untracked();
        let values = slider.values.get_untracked();
        let Some(value) = values.get(index).copied() else {
            return;
        };
        let (min, max, step) = (
            slider.min.get_untracked(),
            slider.max.get_untracked(),
            slider.step.get_untracked(),
        );
        let large = slider.large_step.get_untracked();
        let gap = step * slider.min_steps.get_untracked();
        let back = if slider.direction.get_untracked() == Direction::Rtl {
            1.0
        } else {
            -1.0
        };
        let increment = if event.shift_key() { large } else { step };
        let rounded = round_to_step(value, step, min);
        let nudge = |by: f64, direction: f64| {
            let digits = precision(rounded).max(precision(by)).max(precision(min));
            clamp(fixed(rounded + by * direction, digits), min, max)
        };
        let range = values.len() > 1;
        let key = event.key();
        let next = match key.as_str() {
            "ArrowUp" => nudge(increment, 1.0),
            "ArrowRight" => nudge(increment, -back),
            "ArrowDown" => nudge(increment, -1.0),
            "ArrowLeft" => nudge(increment, back),
            "PageUp" => nudge(large, 1.0),
            "PageDown" => nudge(large, -1.0),
            "End" => match values.get(index + 1) {
                Some(following) if range => following - gap,
                _ => max,
            },
            "Home" => match index.checked_sub(1) {
                Some(previous) if range => values[previous] + gap,
                _ => min,
            },
            _ => return,
        };
        if !key.starts_with("Page") {
            event.stop_propagation();
        }
        slider.input_change(next, index, ChangeReason::Keyboard);
        event.prevent_default();
    };
    view! {
        <div
            {..attrs}
            {..state_attributes(slider)}
            node_ref=element
            data-index=move || index.get().to_string()
            style:position=move || if value().is_some() { "absolute" } else { "fixed" }
            style:inset-inline-start=move || (!vertical()).then(start).flatten()
            style:bottom=move || vertical().then(start).flatten()
            style:top=move || (!vertical()).then_some("50%")
            style:left=move || vertical().then_some("50%")
            style:translate=translate
            style:z-index=z_index
            style:clip-path=move || value().is_none().then_some("inset(50%)")
            on:pointerdown=move |event: ev::PointerEvent| {
                if disabled.get_untracked() {
                    return;
                }
                let Some(thumb) = element.get_untracked() else {
                    return;
                };
                let axis = slider.axis();
                let offset = client(&event, axis) - midpoint(&thumb, axis);
                slider.press.update_value(|press| {
                    press.thumb = Some(index.get_untracked());
                    press.offset = Some(offset);
                });
            }
        >
            {children.map(|children| children())}
            <input
                node_ref=input
                type="range"
                id=thumb.input_id.get()
                aria-label=label
                aria-labelledby=labelled_by
                aria-describedby=move || aria_describedby.map(|described_by| described_by.get())
                aria-orientation=keyword(slider.orientation)
                aria-valuenow=move || value().map(|value| value.to_string())
                aria-valuetext=value_text
                disabled=move || disabled.get()
                form=move || slider.form.map(|form| form.get())
                name=move || slider.name.map(|name| name.get())
                min=move || slider.min.get().to_string()
                max=move || slider.max.get().to_string()
                step=move || slider.step.get().to_string()
                tabindex=move || tabindex.map(|tabindex| tabindex.get())
                value=current
                prop:value=current
                style="clip-path:inset(50%); overflow:hidden; white-space:nowrap; border:0; padding:0; width:100%; height:100%; margin:-1px; position:fixed; top:0; left:0"
                style:writing-mode=move || {
                    vertical()
                        .then(|| if slider.direction.get() == Direction::Rtl { "vertical-rl" } else { "vertical-lr" })
                }
                on:input=move |event| {
                    let value = event_target::<HtmlInputElement>(&event).value_as_number();
                    slider.input_change(value, index.get_untracked(), ChangeReason::InputChange);
                }
                on:focus=move |_| slider.set_active(Some(index.get_untracked()))
                on:blur=move |_| change(slider.active, None)
                on:keydown=keydown
            />
        </div>
    }
}
