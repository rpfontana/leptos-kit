use crate::Keyword;
use leptos::prelude::*;
use web_sys::Element;

pub(crate) const REVERSE_CANCEL_THRESHOLD: f64 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Keyword)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

pub(crate) fn displacement(direction: SwipeDirection, x: f64, y: f64) -> f64 {
    match direction {
        SwipeDirection::Up => -y,
        SwipeDirection::Down => y,
        SwipeDirection::Left => -x,
        SwipeDirection::Right => x,
    }
}

pub(crate) fn transform_of(element: &Element) -> (f64, f64, f64) {
    let Ok(Some(style)) = window().get_computed_style(element) else {
        return (0.0, 0.0, 1.0);
    };
    let transform = style.get_property_value("transform").unwrap_or_default();
    let values: Vec<f64> = transform
        .split_once('(')
        .and_then(|(_, rest)| rest.strip_suffix(')'))
        .map(|inner| {
            inner
                .split(',')
                .filter_map(|value| value.trim().parse().ok())
                .collect()
        })
        .unwrap_or_default();
    match values.as_slice() {
        [a, b, _, _, x, y] => (*x, *y, a.hypot(*b)),
        [scale, .., x, y, _, _] if values.len() == 16 => (*x, *y, *scale),
        _ => (0.0, 0.0, 1.0),
    }
}
