use leptos::prelude::*;
use web_sys::Element;

const COLLISION_PADDING: f64 = 5.0;

// what a positioner tells the popup and the arrow inside it
#[derive(Clone, Copy)]
pub(crate) struct Placement {
    pub(crate) side: Signal<&'static str>,
    pub(crate) align: Signal<&'static str>,
    pub(crate) flipped: RwSignal<bool>,
    nudge: RwSignal<(f64, f64)>,
}

impl Placement {
    pub(crate) fn new(side: Signal<&'static str>, align: Signal<&'static str>) -> Self {
        Self {
            side,
            align,
            flipped: RwSignal::new(false),
            nudge: RwSignal::new((0.0, 0.0)),
        }
    }

    // a layout nudge for the popup, a fraction of a pixel in a translate would blur it
    pub(crate) fn nudge(self, pick: fn((f64, f64)) -> f64) -> impl Fn() -> String + Copy {
        let nudge = self.nudge;
        move || format!("{}px", pick(nudge.get()))
    }

    pub(crate) fn side(&self) -> &'static str {
        let side = self.side.get();
        if self.flipped.get() {
            opposite(side)
        } else {
            side
        }
    }

    // css doesn't say which fallback won, so read the side off the boxes
    pub(crate) fn settle(&self, anchor: &Element, floating: &Element) {
        let Some(side) = self.side.try_get_untracked() else {
            return;
        };
        let (a, f) = (
            anchor.get_bounding_client_rect(),
            floating.get_bounding_client_rect(),
        );
        let flipped = match side {
            "top" => f.top() >= a.bottom() - 1.0,
            "bottom" => f.bottom() <= a.top() + 1.0,
            "left" | "inline-start" => f.left() >= a.right() - 1.0,
            "right" | "inline-end" => f.right() <= a.left() + 1.0,
            _ => false,
        };
        self.flipped.try_set(flipped);
        // floating-ui puts a popup on device pixels, css centring can leave it between two
        let ratio = window().device_pixel_ratio();
        let snap = |at: f64| (at * ratio).round() / ratio - at;
        self.nudge.try_set((snap(f.left()), snap(f.top())));
    }
}

// css anchor positioning for a positioner on one side of its anchor
pub(crate) fn placement(
    anchor: &str,
    side: &str,
    align: &str,
    side_offset: f64,
    align_offset: f64,
) -> String {
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
    let (facing, far) = match side {
        "top" => ("margin-bottom", "margin-top"),
        "left" => ("margin-right", "margin-left"),
        "right" => ("margin-left", "margin-right"),
        "inline-start" => ("margin-inline-end", "margin-inline-start"),
        "inline-end" => ("margin-inline-start", "margin-inline-end"),
        _ => ("margin-top", "margin-bottom"),
    };
    let (start, end) = match side {
        "left" | "right" | "inline-start" | "inline-end" => ("margin-top", "margin-bottom"),
        _ => ("margin-left", "margin-right"),
    };
    // the margin box is what gets placed and kept on screen, so margins give floating-ui's collision padding
    let (before, after) = match align {
        "start" => (align_offset, COLLISION_PADDING),
        "end" => (COLLISION_PADDING, align_offset),
        _ => (
            COLLISION_PADDING + align_offset,
            COLLISION_PADDING - align_offset,
        ),
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
         {facing}:{side_offset}px;{far}:{COLLISION_PADDING}px;{start}:{before}px;{end}:{after}px;\
         --transform-origin:{origin}"
    )
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
pub(crate) fn check(part: &str, side: &str, align: &str) {
    const SIDES: &[&str] = &[
        "top",
        "bottom",
        "left",
        "right",
        "inline-start",
        "inline-end",
    ];
    if !SIDES.contains(&side) {
        leptos::logging::warn!("{part}: unknown side `{side}`, expected one of {SIDES:?}");
    }
    if !["start", "center", "end"].contains(&align) {
        leptos::logging::warn!("{part}: unknown align `{align}`, expected start, center or end");
    }
}
