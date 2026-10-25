use crate::{Align, Side};
use leptos::prelude::*;
use web_sys::Element;

const COLLISION_PADDING: f64 = 5.0;
// as far as a placed popup sits from its anchor
const REACH: f64 = 50.0;

// what a positioner tells the popup and the arrow inside it
#[derive(Clone, Copy)]
pub(crate) struct Placement {
    pub(crate) side: Signal<Side>,
    pub(crate) align: Signal<Align>,
    pub(crate) flipped: RwSignal<bool>,
    nudge: RwSignal<(f64, f64)>,
    sizes: RwSignal<Option<[f64; 4]>>,
}

impl Placement {
    pub(crate) fn new(side: Signal<Side>, align: Signal<Align>) -> Self {
        Self {
            side,
            align,
            flipped: RwSignal::new(false),
            nudge: RwSignal::new((0.0, 0.0)),
            sizes: RwSignal::new(None),
        }
    }

    // base ui's css variables for the popup, known once it's placed
    pub(crate) fn sizes(self) -> impl Fn() -> String + Copy {
        let sizes = self.sizes;
        move || {
            sizes
                .get()
                .map_or_else(String::new, |[width, height, across, along]| {
                    format!(
                        "--anchor-width:{width}px;--anchor-height:{height}px;\
                     --available-width:{across}px;--available-height:{along}px"
                    )
                })
        }
    }

    // a layout nudge for the popup, a fraction of a pixel in a translate would blur it
    pub(crate) fn nudge(self, pick: fn((f64, f64)) -> f64) -> impl Fn() -> String + Copy {
        let nudge = self.nudge;
        move || format!("{}px", pick(nudge.get()))
    }

    pub(crate) fn side(&self) -> Side {
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
        // webkit places a popup anchored inside the top layer as if the page weren't scrolled
        let near = |dx: f64, dy: f64| {
            f.left() + dx <= a.right() + REACH
                && f.right() + dx >= a.left() - REACH
                && f.top() + dy <= a.bottom() + REACH
                && f.bottom() + dy >= a.top() - REACH
        };
        let scroll = (
            window().scroll_x().unwrap_or(0.0),
            window().scroll_y().unwrap_or(0.0),
        );
        let (dx, dy) = if !near(0.0, 0.0) && near(scroll.0, scroll.1) {
            scroll
        } else {
            (0.0, 0.0)
        };
        let (left, top, right, bottom) =
            (f.left() + dx, f.top() + dy, f.right() + dx, f.bottom() + dy);
        let flipped = match side {
            Side::Top => top >= a.bottom() - 1.0,
            Side::Bottom => bottom <= a.top() + 1.0,
            Side::Left | Side::InlineStart => left >= a.right() - 1.0,
            Side::Right | Side::InlineEnd => right <= a.left() + 1.0,
        };
        self.flipped.try_set(flipped);
        // floating-ui puts a popup on device pixels, css centring can leave it between two
        let ratio = window().device_pixel_ratio();
        let snap = |at: f64| (at * ratio).round() / ratio;
        self.nudge
            .try_set((snap(left) - f.left(), snap(top) - f.top()));
        // floating-ui's size middleware: the room on the side it's on, the whole viewport across
        let Some(viewport) = document().document_element() else {
            return;
        };
        let (width, height) = (
            f64::from(viewport.client_width()),
            f64::from(viewport.client_height()),
        );
        let padding = COLLISION_PADDING;
        let shown = if flipped { opposite(side) } else { side };
        let (across, along) = match shown {
            Side::Top => (width - 2.0 * padding, bottom - padding),
            Side::Bottom => (width - 2.0 * padding, height - padding - top),
            Side::Left | Side::InlineStart => (right - padding, height - 2.0 * padding),
            Side::Right | Side::InlineEnd => (width - padding - left, height - 2.0 * padding),
        };
        self.sizes
            .try_set(Some([a.width(), a.height(), across, along]));
    }
}

// css anchor positioning for a positioner on one side of its anchor
pub(crate) fn placement(
    anchor: &str,
    side: Side,
    align: Align,
    side_offset: f64,
    align_offset: f64,
) -> String {
    let (span_start, span_end, fallback) = match side {
        Side::Top | Side::Bottom => ("span-right", "span-left", "flip-block"),
        Side::Left | Side::Right => ("span-bottom", "span-top", "flip-inline"),
        Side::InlineStart | Side::InlineEnd => {
            ("span-block-end", "span-block-start", "flip-inline")
        }
    };
    let block = side.as_str();
    let area = match align {
        Align::Start => format!("{block} {span_start}"),
        Align::End => format!("{block} {span_end}"),
        Align::Center => block.to_string(),
    };
    // floating-ui flips the alignment too, start to end, when the side alone doesn't fit
    let fallback = match (align, fallback) {
        (Align::Start | Align::End, "flip-block") => {
            "flip-block, flip-inline, flip-block flip-inline"
        }
        (Align::Start | Align::End, _) => "flip-inline, flip-block, flip-inline flip-block",
        (Align::Center, _) => fallback,
    };
    let (facing, far) = match side {
        Side::Top => ("margin-bottom", "margin-top"),
        Side::Left => ("margin-right", "margin-left"),
        Side::Right => ("margin-left", "margin-right"),
        Side::InlineStart => ("margin-inline-end", "margin-inline-start"),
        Side::InlineEnd => ("margin-inline-start", "margin-inline-end"),
        Side::Bottom => ("margin-top", "margin-bottom"),
    };
    let (start, end) = match side {
        Side::Left | Side::Right | Side::InlineStart | Side::InlineEnd => {
            ("margin-top", "margin-bottom")
        }
        Side::Top | Side::Bottom => ("margin-left", "margin-right"),
    };
    // the margin box is what gets placed and kept on screen, so margins give floating-ui's collision padding
    let (before, after) = match align {
        Align::Start => (align_offset, COLLISION_PADDING),
        Align::End => (COLLISION_PADDING, align_offset),
        Align::Center => (
            COLLISION_PADDING + align_offset,
            COLLISION_PADDING - align_offset,
        ),
    };
    let origin = match side {
        Side::Top => "bottom",
        Side::Left | Side::InlineStart => "right",
        Side::Right | Side::InlineEnd => "left",
        Side::Bottom => "top",
    };
    format!(
        "inset:auto;margin:0;padding:0;border:0;background:transparent;color:inherit;overflow:visible;         position-anchor:{anchor};position-area:{area};position-try-fallbacks:{fallback};         {facing}:{side_offset}px;{far}:{COLLISION_PADDING}px;{start}:{before}px;{end}:{after}px;         --transform-origin:{origin}"
    )
}

fn opposite(side: Side) -> Side {
    match side {
        Side::Top => Side::Bottom,
        Side::Bottom => Side::Top,
        Side::Left => Side::Right,
        Side::Right => Side::Left,
        Side::InlineStart => Side::InlineEnd,
        Side::InlineEnd => Side::InlineStart,
    }
}
