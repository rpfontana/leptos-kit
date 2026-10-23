use leptos::ev;
use leptos::prelude::*;
use std::time::Duration;
use web_sys::{DomRect, Element};

// base ui's hover interaction: open after a delay, close after another, and a way across to the popup
#[derive(Clone, Copy)]
pub(crate) struct Hover {
    opening: StoredValue<Option<TimeoutHandle>>,
    closing: StoredValue<Option<TimeoutHandle>>,
    crossing: StoredValue<Option<WindowListenerHandle>>,
}

impl Hover {
    pub(crate) fn new() -> Self {
        let this = Self {
            opening: StoredValue::new(None),
            closing: StoredValue::new(None),
            crossing: StoredValue::new(None),
        };
        on_cleanup(move || this.cancel());
        this
    }

    pub(crate) fn cancel(self) {
        stop(self.opening);
        stop(self.closing);
        self.stop_crossing();
    }

    pub(crate) fn enter(self, delay: u32, open: Callback<()>) {
        stop(self.closing);
        self.stop_crossing();
        schedule(self.opening, delay, open);
    }

    // over the popup nothing closes it
    pub(crate) fn hold(self) {
        stop(self.closing);
        self.stop_crossing();
    }

    pub(crate) fn leave(self, delay: u32, close: Callback<()>) {
        stop(self.opening);
        self.stop_crossing();
        schedule(self.closing, delay, close);
    }

    // leaving one box for the other keeps it open while the pointer stays on the way between them
    pub(crate) fn cross(
        self,
        from: (f64, f64),
        boxes: [Element; 2],
        delay: u32,
        close: Callback<()>,
    ) {
        stop(self.opening);
        self.stop_crossing();
        if !between(&boxes, from) {
            schedule(self.closing, delay, close);
            return;
        }
        let handle = window_event_listener(ev::pointermove, move |event| {
            let at = (f64::from(event.client_x()), f64::from(event.client_y()));
            let over = boxes
                .iter()
                .any(|element| inside(&element.get_bounding_client_rect(), at));
            if over {
                self.stop_crossing();
            } else if !between(&boxes, at) {
                self.stop_crossing();
                schedule(self.closing, delay, close);
            }
        });
        self.crossing.try_set_value(Some(handle));
    }

    fn stop_crossing(self) {
        if let Some(Some(handle)) = self.crossing.try_update_value(Option::take) {
            handle.remove();
        }
    }
}

fn stop(timer: StoredValue<Option<TimeoutHandle>>) {
    if let Some(Some(handle)) = timer.try_update_value(Option::take) {
        handle.clear();
    }
}

fn schedule(timer: StoredValue<Option<TimeoutHandle>>, delay: u32, run: Callback<()>) {
    stop(timer);
    if delay == 0 {
        run.run(());
        return;
    }
    let handle = set_timeout_with_handle(move || run.run(()), Duration::from_millis(delay.into()));
    timer.try_set_value(handle.ok());
}

fn inside(rect: &DomRect, (x, y): (f64, f64)) -> bool {
    x >= rect.left() && x <= rect.right() && y >= rect.top() && y <= rect.bottom()
}

// the trapezoid from one box's facing edge to the other's, whichever side they meet on
fn between([a, b]: &[Element; 2], at: (f64, f64)) -> bool {
    let (a, b) = (a.get_bounding_client_rect(), b.get_bounding_client_rect());
    let (near, far) = if b.bottom() <= a.top() + 1.0 || b.right() <= a.left() + 1.0 {
        (b, a)
    } else {
        (a, b)
    };
    let quad = if far.top() >= near.bottom() - 1.0 {
        [
            (near.left(), near.bottom()),
            (near.right(), near.bottom()),
            (far.right(), far.top()),
            (far.left(), far.top()),
        ]
    } else {
        [
            (near.right(), near.top()),
            (near.right(), near.bottom()),
            (far.left(), far.bottom()),
            (far.left(), far.top()),
        ]
    };
    convex(&quad, at)
}

fn convex(points: &[(f64, f64); 4], (x, y): (f64, f64)) -> bool {
    let mut sign = 0.0;
    for (index, &(ax, ay)) in points.iter().enumerate() {
        let (bx, by) = points[(index + 1) % points.len()];
        let cross = (bx - ax) * (y - ay) - (by - ay) * (x - ax);
        if cross == 0.0 {
            continue;
        }
        if sign == 0.0 {
            sign = cross.signum();
        } else if cross.signum() != sign {
            return false;
        }
    }
    true
}
