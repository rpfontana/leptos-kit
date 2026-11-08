use crate::Keyword;
use crate::id::ElementId;
use crate::native::{document_order, observe_resize};
use crate::{Orientation, use_id};
use leptos::context::Provider;
use leptos::either::Either;
use leptos::ev;
use leptos::html::Div;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_kit_macros::component;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use web_sys::{CssStyleSheet, FocusOptions, HtmlElement};

const KEYBOARD_STEP: f64 = 5.0;
const HORIZONTAL_MIN: u8 = 0b0001;
const HORIZONTAL_MAX: u8 = 0b0010;
const VERTICAL_MIN: u8 = 0b0100;
const VERTICAL_MAX: u8 = 0b1000;
const HORIZONTAL: u8 = 0b0011;
const VERTICAL: u8 = 0b1100;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Percent(f64),
    Px(f64),
    Em(f64),
    Rem(f64),
    Vh(f64),
    Vw(f64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseLengthError;

impl fmt::Display for ParseLengthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid panel size, expected a number in %, px, em, rem, vh or vw")
    }
}

impl std::error::Error for ParseLengthError {}

impl FromStr for Length {
    type Err = ParseLengthError;

    // a bare number is a percentage, as react-resizable-panels reads a string
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        type Unit = fn(f64) -> Length;
        let units: [(&str, Unit); 6] = [
            ("%", Length::Percent),
            ("px", Length::Px),
            ("rem", Length::Rem),
            ("em", Length::Em),
            ("vh", Length::Vh),
            ("vw", Length::Vw),
        ];
        let text = text.trim();
        let (number, unit) = units
            .iter()
            .find_map(|(suffix, unit)| text.strip_suffix(suffix).map(|number| (number, *unit)))
            .unwrap_or((text, Length::Percent));
        number
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|number| number.is_finite())
            .map(unit)
            .ok_or(ParseLengthError)
    }
}

// panics on a size it can't read, `str::parse` doesn't
impl From<&str> for Length {
    fn from(text: &str) -> Self {
        text.parse()
            .unwrap_or_else(|_| panic!("`{text}` is not a panel size"))
    }
}

impl From<String> for Length {
    fn from(text: String) -> Self {
        Self::from(text.as_str())
    }
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Percent(value) => write!(f, "{value}%"),
            Self::Px(value) => write!(f, "{value}px"),
            Self::Em(value) => write!(f, "{value}em"),
            Self::Rem(value) => write!(f, "{value}rem"),
            Self::Vh(value) => write!(f, "{value}vh"),
            Self::Vw(value) => write!(f, "{value}vw"),
        }
    }
}

impl Length {
    fn pixels(self, group: f64, panel: Option<&HtmlElement>) -> f64 {
        match self {
            Self::Percent(value) => (value / 100.0) * group,
            Self::Px(value) => value,
            Self::Rem(value) => {
                value
                    * document()
                        .document_element()
                        .map_or(0.0, |root| font_size(&root))
            }
            Self::Em(value) => value * panel.map_or(0.0, |panel| font_size(panel)),
            Self::Vh(value) => (value / 100.0) * viewport(web_sys::Window::inner_height),
            Self::Vw(value) => (value / 100.0) * viewport(web_sys::Window::inner_width),
        }
    }

    fn share(self, group: Option<f64>, panel: Option<&HtmlElement>) -> Option<f64> {
        match (self, group) {
            (_, Some(group)) => Some(fixed((self.pixels(group, panel) / group) * 100.0)),
            (Self::Percent(value), None) => Some(fixed(value)),
            _ => None,
        }
    }
}

fn font_size(element: &web_sys::Element) -> f64 {
    window()
        .get_computed_style(element)
        .ok()
        .flatten()
        .and_then(|style| style.get_property_value("font-size").ok())
        .and_then(|size| size.trim_end_matches("px").parse().ok())
        .unwrap_or(0.0)
}

fn viewport(
    size: impl Fn(
        &web_sys::Window,
    ) -> Result<leptos::wasm_bindgen::JsValue, leptos::wasm_bindgen::JsValue>,
) -> f64 {
    size(&window())
        .ok()
        .and_then(|size| size.as_f64())
        .unwrap_or(0.0)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Keyword)]
pub enum GroupResizeBehavior {
    #[default]
    PreserveRelativeSize,
    PreservePixelSize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PanelSize {
    pub as_percentage: f64,
    pub in_pixels: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LayoutChangedMeta {
    pub is_user_interaction: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layout(Vec<(String, f64)>);

impl Layout {
    pub fn get(&self, id: &str) -> Option<f64> {
        self.0
            .iter()
            .find_map(|(key, size)| (key == id).then_some(*size))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, f64)> {
        self.0.iter().map(|(id, size)| (id.as_str(), *size))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn same(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .all(|(id, size)| other.get(id).is_some_and(|other| equal(size, other)))
    }
}

impl<K: Into<String>> FromIterator<(K, f64)> for Layout {
    fn from_iter<I: IntoIterator<Item = (K, f64)>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|(id, size)| (id.into(), size))
                .collect(),
        )
    }
}

impl<K: Into<String>, const N: usize> From<[(K, f64); N]> for Layout {
    fn from(sizes: [(K, f64); N]) -> Self {
        sizes.into_iter().collect()
    }
}

impl IntoIterator for Layout {
    type Item = (String, f64);
    type IntoIter = std::vec::IntoIter<(String, f64)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Keyword)]
enum SeparatorState {
    Inactive,
    Hover,
    Active,
    Focus,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Constraints {
    collapsed_size: f64,
    collapsible: bool,
    default_size: Option<f64>,
    disabled: bool,
    min_size: f64,
    max_size: f64,
    behavior: GroupResizeBehavior,
}

#[derive(Clone, Copy)]
struct PanelEntry {
    key: u64,
    id: ElementId<Div>,
    node: NodeRef<Div>,
    default_size: Option<Length>,
    min_size: Option<Length>,
    max_size: Option<Length>,
    collapsed_size: Option<Length>,
    collapsible: bool,
    disabled: Signal<bool>,
    behavior: GroupResizeBehavior,
    expand_to: StoredValue<Option<f64>>,
}

impl PanelEntry {
    fn constraints(&self, group: Option<f64>) -> Constraints {
        let element = group.and_then(|_| self.node.get_untracked());
        let element = element.as_deref();
        let share = |length: Option<Length>| length.and_then(|length| length.share(group, element));
        Constraints {
            collapsed_size: share(self.collapsed_size).unwrap_or(0.0),
            collapsible: self.collapsible,
            default_size: share(self.default_size),
            disabled: self.disabled.get_untracked(),
            min_size: share(self.min_size).unwrap_or(0.0),
            max_size: share(self.max_size).unwrap_or(100.0),
            behavior: self.behavior,
        }
    }
}

#[derive(Clone, Copy)]
struct SeparatorEntry {
    key: u64,
    node: NodeRef<Div>,
    disabled: Signal<bool>,
    disable_double_click: bool,
    hovered: RwSignal<bool>,
}

#[derive(Clone, Copy)]
enum Child {
    Panel(PanelEntry),
    Separator(SeparatorEntry),
}

impl Child {
    fn key(&self) -> u64 {
        match self {
            Self::Panel(panel) => panel.key,
            Self::Separator(separator) => separator.key,
        }
    }

    fn node(&self) -> NodeRef<Div> {
        match self {
            Self::Panel(panel) => panel.node,
            Self::Separator(separator) => separator.node,
        }
    }
}

fn panels(children: &[Child]) -> Vec<PanelEntry> {
    children
        .iter()
        .filter_map(|child| match child {
            Child::Panel(panel) => Some(*panel),
            Child::Separator(_) => None,
        })
        .collect()
}

// the panels either side of a separator that moves them, as react-resizable-panels finds its hit regions
fn pivots(children: &[Child], separator: u64) -> Option<[usize; 2]> {
    let panels = panels(children);
    let enabled: Vec<usize> = panels
        .iter()
        .enumerate()
        .filter_map(|(index, panel)| (!panel.disabled.get()).then_some(index))
        .collect();
    let (&first, &last) = (enabled.first()?, enabled.last()?);
    if enabled.len() < 2 {
        return None;
    }
    let mut seen: Option<usize> = None;
    let mut before = None;
    for child in children {
        match child {
            Child::Panel(_) => seen = Some(seen.map_or(0, |index| index + 1)),
            Child::Separator(entry) if entry.key == separator => {
                if entry.disabled.get() {
                    return None;
                }
                before = Some(seen);
            }
            Child::Separator(_) => {}
        }
    }
    let previous = before??;
    let next = previous + 1;
    (next < panels.len() && next > first && next <= last).then_some([previous, next])
}

#[derive(Clone, Debug, PartialEq)]
struct Measured {
    layout: Vec<f64>,
    constraints: Vec<Constraints>,
    size: f64,
}

#[derive(Clone, Debug)]
struct Drag {
    separator: u64,
    pointer: i32,
    from: (f64, f64),
    initial: Vec<f64>,
    size: f64,
    pivots: [usize; 2],
    rtl: bool,
    flags: u8,
}

#[derive(Clone, Debug, Default)]
struct Reported {
    change: Option<Layout>,
    changed: Option<Layout>,
}

#[derive(Clone, Copy)]
struct Group {
    orientation: Signal<Orientation>,
    disabled: Signal<bool>,
    disable_cursor: Signal<bool>,
    node: NodeRef<Div>,
    children: RwSignal<Vec<Child>>,
    keys: StoredValue<u64>,
    measured: RwSignal<Option<Measured>>,
    default_layout: StoredValue<Option<Layout>>,
    layouts: StoredValue<HashMap<String, Vec<f64>>>,
    expanded: StoredValue<HashMap<u64, f64>>,
    drag: StoredValue<Option<Drag>>,
    active: RwSignal<Option<u64>>,
    reported: StoredValue<Reported>,
    on_layout_change: Option<Callback<Layout>>,
    on_layout_changed: Option<Callback<(Layout, LayoutChangedMeta)>>,
}

#[derive(Clone, Copy)]
struct Resizable(StoredValue<Group>);

fn use_resizable(part: &str) -> Resizable {
    use_context::<Resizable>().unwrap_or_else(|| panic!("a {part} goes inside a ResizableGroup"))
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Flex {
    grow: String,
    basis: String,
}

#[derive(Clone, Debug, PartialEq)]
struct Aria {
    controls: String,
    min: f64,
    max: f64,
    now: f64,
}

enum Estimate {
    Grow(Vec<f64>),
    // a size in another unit waits for the group's, as react-resizable-panels renders it on the server
    Basis,
}

impl Resizable {
    fn group(self) -> Option<Group> {
        self.0.try_get_value()
    }

    fn register(self, child: impl FnOnce(u64) -> Child) -> u64 {
        let Some(group) = self.group() else {
            return 0;
        };
        let key = group
            .keys
            .try_update_value(|next| {
                *next += 1;
                *next
            })
            .unwrap_or_default();
        group.children.update(|children| children.push(child(key)));
        key
    }

    fn unregister(self, key: u64) {
        if let Some(group) = self.group() {
            group
                .children
                .try_update(|children| children.retain(|child| child.key() != key));
        }
    }

    fn horizontal(group: &Group) -> bool {
        group.orientation.get_untracked() == Orientation::Horizontal
    }

    fn measure(panels: &[PanelEntry], horizontal: bool) -> f64 {
        panels
            .iter()
            .filter_map(|panel| panel.node.get_untracked())
            .map(|element| {
                f64::from(if horizontal {
                    element.offset_width()
                } else {
                    element.offset_height()
                })
            })
            .sum()
    }

    fn estimate(group: &Group, children: &[Child]) -> Estimate {
        let panels = panels(children);
        let stated = panels
            .iter()
            .all(|panel| matches!(panel.default_size, None | Some(Length::Percent(_))));
        if !stated {
            return Estimate::Basis;
        }
        let constraints: Vec<Constraints> =
            panels.iter().map(|panel| panel.constraints(None)).collect();
        let layout = group
            .default_layout
            .with_value(|layout| layout.as_ref().and_then(|layout| ordered(layout, &panels)))
            .unwrap_or_else(|| default_layout(&constraints));
        Estimate::Grow(validate_layout(&layout, &constraints))
    }

    fn flex(self, key: u64) -> Flex {
        let Some(group) = self.group() else {
            return Flex::default();
        };
        group.children.with(|children| {
            let panels = panels(children);
            let index = panels.iter().position(|panel| panel.key == key);
            let grow = |size: f64| Flex {
                grow: number(size),
                basis: "0".into(),
            };
            let measured = group.measured.with(|measured| {
                measured
                    .as_ref()
                    .filter(|measured| measured.layout.len() == panels.len())
                    .and_then(|measured| {
                        index.and_then(|index| measured.layout.get(index).copied())
                    })
            });
            if let Some(size) = measured {
                return grow(size);
            }
            match (Self::estimate(&group, children), index) {
                (Estimate::Grow(layout), Some(index)) => grow(layout[index]),
                (Estimate::Basis, Some(index)) => match panels[index].default_size {
                    Some(size) => Flex {
                        grow: "0".into(),
                        basis: size.to_string(),
                    },
                    None => grow(1.0),
                },
                (_, None) => grow(1.0),
            }
        })
    }

    fn aria(self, key: u64) -> Option<Aria> {
        let group = self.group()?;
        group.children.with(|children| {
            let [primary, _] = pivots(children, key)?;
            let panels = panels(children);
            let measured = group.measured.with(|measured| {
                measured
                    .as_ref()
                    .filter(|measured| measured.layout.len() == panels.len())
                    .map(|measured| {
                        let constraints = measured
                            .constraints
                            .iter()
                            .zip(&panels)
                            .map(|(constraints, panel)| Constraints {
                                disabled: panel.disabled.get(),
                                ..*constraints
                            })
                            .collect::<Vec<_>>();
                        (measured.layout.clone(), constraints)
                    })
            });
            let (layout, constraints) = match measured {
                Some(measured) => measured,
                None => match Self::estimate(&group, children) {
                    Estimate::Grow(layout) => (
                        layout,
                        panels.iter().map(|panel| panel.constraints(None)).collect(),
                    ),
                    Estimate::Basis => return None,
                },
            };
            let (min, max) = aria_values(&layout, &constraints, primary);
            Some(Aria {
                controls: panels[primary].id.get(),
                min,
                max,
                now: layout[primary],
            })
        })
    }

    fn named(panels: &[PanelEntry], layout: &[f64]) -> Layout {
        panels
            .iter()
            .zip(layout)
            .map(|(panel, size)| (panel.id.get(), *size))
            .collect()
    }

    fn layout(self) -> Layout {
        let Some(group) = self.group() else {
            return Layout::default();
        };
        group.children.with(|children| {
            let panels = panels(children);
            group.measured.with(|measured| {
                measured
                    .as_ref()
                    .filter(|measured| measured.layout.len() == panels.len())
                    .map(|measured| Self::named(&panels, &measured.layout))
                    .unwrap_or_default()
            })
        })
    }

    fn current(group: &Group) -> Option<(Vec<PanelEntry>, Measured)> {
        let panels = group.children.with_untracked(|children| panels(children));
        let mut measured = group.measured.get_untracked()?;
        if measured.layout.len() != panels.len() {
            return None;
        }
        for (constraints, panel) in measured.constraints.iter_mut().zip(&panels) {
            constraints.disabled = panel.disabled.get_untracked();
        }
        Some((panels, measured))
    }

    fn mount(self) {
        let Some(group) = self.group() else {
            return;
        };
        let children = group.children.get_untracked();
        let nodes: Option<Vec<HtmlElement>> = children
            .iter()
            .map(|child| child.node().get_untracked().map(Into::into))
            .collect();
        let Some(nodes) = nodes else {
            return;
        };
        // a panel shown later registers last, the page's order is the one that counts
        let mut sorted: Vec<(Child, HtmlElement)> = children.iter().copied().zip(nodes).collect();
        sorted.sort_by(|a, b| document_order(&a.1, &b.1));
        if sorted
            .iter()
            .zip(&children)
            .any(|((a, _), b)| a.key() != b.key())
        {
            group
                .children
                .set(sorted.into_iter().map(|(child, _)| child).collect());
            return;
        }
        let panels = panels(&children);
        let size = Self::measure(&panels, Self::horizontal(&group));
        // in a hidden subtree, so the observer measures it once it shows
        if size == 0.0 {
            return;
        }
        let constraints: Vec<Constraints> = panels
            .iter()
            .map(|panel| panel.constraints(Some(size)))
            .collect();
        let ids = ids(&panels);
        let layout = group
            .layouts
            .with_value(|layouts| layouts.get(&ids).cloned())
            .or_else(|| {
                group.default_layout.with_value(|layout| {
                    layout.as_ref().and_then(|layout| ordered(layout, &panels))
                })
            })
            .unwrap_or_else(|| default_layout(&constraints));
        let layout = validate_layout(&layout, &constraints);
        group.measured.set(Some(Measured {
            layout,
            constraints,
            size,
        }));
        self.report(false);
    }

    fn resized(self) {
        let Some(group) = self.group() else {
            return;
        };
        let panels = group.children.with_untracked(|children| panels(children));
        let size = Self::measure(&panels, Self::horizontal(&group));
        if size == 0.0 {
            return;
        }
        let Some(previous) = group
            .measured
            .get_untracked()
            .filter(|measured| measured.layout.len() == panels.len())
        else {
            self.mount();
            return;
        };
        let constraints: Vec<Constraints> = panels
            .iter()
            .map(|panel| panel.constraints(Some(size)))
            .collect();
        let layout = validate_layout(
            &preserve_fixed(&constraints, size, previous.size, &previous.layout),
            &constraints,
        );
        if same(&layout, &previous.layout)
            && constraints == previous.constraints
            && size == previous.size
        {
            return;
        }
        self.update(
            Measured {
                layout,
                constraints,
                size,
            },
            false,
        );
    }

    fn update(self, next: Measured, user: bool) {
        let Some(group) = self.group() else {
            return;
        };
        let panels = group.children.with_untracked(|children| panels(children));
        if let Some(previous) = group.measured.get_untracked()
            && previous.layout.len() == next.layout.len()
        {
            for (index, constraints) in next.constraints.iter().enumerate() {
                if constraints.collapsible
                    && equal(constraints.collapsed_size, next.layout[index])
                    && !equal(constraints.collapsed_size, previous.layout[index])
                {
                    group.expanded.update_value(|expanded| {
                        expanded.insert(panels[index].key, previous.layout[index]);
                    });
                }
            }
        }
        group.layouts.update_value(|layouts| {
            layouts.insert(ids(&panels), next.layout.clone());
        });
        group.measured.set(Some(next));
        self.report(user);
    }

    fn commit(self, layout: Vec<f64>, user: bool) {
        let Some(group) = self.group() else {
            return;
        };
        if let Some(measured) = group.measured.get_untracked() {
            self.update(Measured { layout, ..measured }, user);
        }
    }

    fn report(self, user: bool) {
        let Some(group) = self.group() else {
            return;
        };
        let layout = untrack(|| self.layout());
        let fresh = |last: &mut Option<Layout>| {
            let fresh = !last.as_ref().is_some_and(|last| last.same(&layout));
            if fresh {
                *last = Some(layout.clone());
            }
            fresh
        };
        if group
            .reported
            .try_update_value(|reported| fresh(&mut reported.change))
            .unwrap_or(false)
            && let Some(callback) = group.on_layout_change
        {
            callback.run(layout.clone());
        }
        let dragging = group.drag.with_value(Option::is_some);
        if !dragging
            && group
                .reported
                .try_update_value(|reported| fresh(&mut reported.changed))
                .unwrap_or(false)
            && let Some(callback) = group.on_layout_changed
        {
            callback.run((
                layout,
                LayoutChangedMeta {
                    is_user_interaction: user,
                },
            ));
        }
    }

    fn usable(self, separator: u64) -> Option<[usize; 2]> {
        let group = self.group()?;
        if group.disabled.get_untracked() {
            return None;
        }
        untrack(|| group.children.with(|children| pivots(children, separator)))
    }

    fn rtl(group: &Group) -> bool {
        Self::horizontal(group)
            && group.node.get_untracked().is_some_and(|element| {
                window()
                    .get_computed_style(&element)
                    .ok()
                    .flatten()
                    .and_then(|style| style.get_property_value("direction").ok())
                    .is_some_and(|direction| direction == "rtl")
            })
    }

    fn hover_cursor(self) {
        if let Some(group) = self.group()
            && !group.disable_cursor.get_untracked()
        {
            show_cursor(Some(cursor(group.orientation.get_untracked(), 0)));
        }
    }

    fn enter(self, separator: SeparatorEntry) {
        let Some(group) = self.group() else {
            return;
        };
        if self.usable(separator.key).is_none() {
            return;
        }
        separator.hovered.set(true);
        if group.active.get_untracked().is_none() {
            self.hover_cursor();
        }
    }

    fn leave(self, separator: SeparatorEntry) {
        let Some(group) = self.group() else {
            return;
        };
        separator.hovered.set(false);
        if group.active.get_untracked().is_none() {
            show_cursor(None);
        }
    }

    fn pointer_down(self, separator: SeparatorEntry, event: &ev::PointerEvent) {
        if event.default_prevented() || (event.pointer_type() == "mouse" && event.button() > 0) {
            return;
        }
        let (Some(group), Some(pivots)) = (self.group(), self.usable(separator.key)) else {
            return;
        };
        let Some((_, measured)) = Self::current(&group) else {
            return;
        };
        let Some(element) = event
            .current_target()
            .and_then(|target| target.dyn_into::<HtmlElement>().ok())
        else {
            return;
        };
        let options = FocusOptions::new();
        options.set_prevent_scroll(true);
        options.set_focus_visible(false);
        let _ = element.focus_with_options(&options);
        let _ = element.set_pointer_capture(event.pointer_id());
        group.drag.set_value(Some(Drag {
            separator: separator.key,
            pointer: event.pointer_id(),
            from: (f64::from(event.client_x()), f64::from(event.client_y())),
            initial: measured.layout,
            size: measured.size,
            pivots,
            rtl: Self::rtl(&group),
            flags: 0,
        }));
        group.active.set(Some(separator.key));
        event.prevent_default();
        self.hover_cursor();
    }

    fn pointer_move(self, event: &ev::PointerEvent) {
        let Some(group) = self.group() else {
            return;
        };
        let Some(drag) = group
            .drag
            .get_value()
            .filter(|drag| drag.pointer == event.pointer_id())
        else {
            return;
        };
        // released over another document, an iframe's, where the pointerup never came
        if event.buttons() == 0 {
            self.end();
            return;
        }
        let Some((_, measured)) = Self::current(&group) else {
            return;
        };
        let orientation = group.orientation.get_untracked();
        let moved = match orientation {
            Orientation::Horizontal => f64::from(event.client_x()) - drag.from.0,
            Orientation::Vertical => f64::from(event.client_y()) - drag.from.1,
        };
        // panels keep the page's order, so right to left the first one grows as the pointer goes left
        let ahead = if drag.rtl { -moved } else { moved };
        let delta = (ahead / drag.size) * 100.0;
        let layout = adjust(
            delta,
            &drag.initial,
            &measured.constraints,
            drag.pivots,
            &measured.layout,
            Trigger::Pointer,
        );
        let mut flags = 0;
        if same(&layout, &measured.layout) {
            if delta != 0.0 && !group.disable_cursor.get_untracked() {
                flags = match (orientation, moved < 0.0) {
                    (Orientation::Horizontal, true) => HORIZONTAL_MIN,
                    (Orientation::Horizontal, false) => HORIZONTAL_MAX,
                    (Orientation::Vertical, true) => VERTICAL_MIN,
                    (Orientation::Vertical, false) => VERTICAL_MAX,
                };
            }
        } else {
            self.commit(layout, true);
        }
        // firefox rounds the pointer's position, so a step that didn't move keeps the last cursor
        let horizontal = if event.movement_x() == 0 {
            drag.flags
        } else {
            flags
        };
        let vertical = if event.movement_y() == 0 {
            drag.flags
        } else {
            flags
        };
        let flags = (horizontal & HORIZONTAL) | (vertical & VERTICAL);
        group.drag.update_value(|drag| {
            if let Some(drag) = drag {
                drag.flags = flags;
            }
        });
        if !group.disable_cursor.get_untracked() {
            show_cursor(Some(cursor(orientation, flags)));
        }
    }

    fn end(self) {
        let Some(group) = self.group() else {
            return;
        };
        let Some(drag) = group.drag.try_update_value(Option::take).flatten() else {
            return;
        };
        group.active.set(None);
        let separator = group.children.with_untracked(|children| {
            children.iter().find_map(|child| match child {
                Child::Separator(separator) if separator.key == drag.separator => Some(*separator),
                _ => None,
            })
        });
        if let Some(element) = separator.and_then(|separator| separator.node.get_untracked())
            && element.has_pointer_capture(drag.pointer)
        {
            let _ = element.release_pointer_capture(drag.pointer);
        }
        self.report(true);
        if separator.is_some_and(|separator| separator.hovered.get_untracked()) {
            self.hover_cursor();
        } else {
            show_cursor(None);
        }
    }

    fn step(self, separator: u64, delta: f64) {
        let (Some(group), Some(pivots)) = (self.group(), self.usable(separator)) else {
            return;
        };
        let Some((_, measured)) = Self::current(&group) else {
            return;
        };
        let layout = validate_layout(
            &adjust(
                delta,
                &measured.layout,
                &measured.constraints,
                pivots,
                &measured.layout,
                Trigger::Keyboard,
            ),
            &measured.constraints,
        );
        if !same(&layout, &measured.layout) {
            self.commit(layout, true);
        }
    }

    fn key(self, separator: SeparatorEntry, event: &ev::KeyboardEvent) {
        let Some(group) = self.group() else {
            return;
        };
        if event.default_prevented() || group.disabled.get_untracked() {
            return;
        }
        let horizontal = Self::horizontal(&group);
        // right to left the arrows still move the separator the way they point
        let ahead = if Self::rtl(&group) {
            -KEYBOARD_STEP
        } else {
            KEYBOARD_STEP
        };
        let delta = match event.key().as_str() {
            "ArrowDown" => (!horizontal).then_some(KEYBOARD_STEP),
            "ArrowUp" => (!horizontal).then_some(-KEYBOARD_STEP),
            "ArrowRight" => horizontal.then_some(ahead),
            "ArrowLeft" => horizontal.then_some(-ahead),
            "End" => Some(100.0),
            "Home" => Some(-100.0),
            "Enter" => {
                event.prevent_default();
                self.toggle(separator.key);
                return;
            }
            "F6" => {
                event.prevent_default();
                self.cycle(separator.key, event.shift_key());
                return;
            }
            _ => return,
        };
        event.prevent_default();
        if let Some(delta) = delta {
            self.step(separator.key, delta);
        }
    }

    fn toggle(self, separator: u64) {
        let (Some(group), Some([primary, _])) = (self.group(), self.usable(separator)) else {
            return;
        };
        let Some((panels, measured)) = Self::current(&group) else {
            return;
        };
        let constraints = measured.constraints[primary];
        if !constraints.collapsible {
            return;
        }
        let size = measured.layout[primary];
        let next = if constraints.collapsed_size == size {
            group
                .expanded
                .with_value(|expanded| expanded.get(&panels[primary].key).copied())
                .unwrap_or(constraints.min_size)
        } else {
            constraints.collapsed_size
        };
        self.step(separator, next - size);
    }

    fn cycle(self, separator: u64, back: bool) {
        let Some(group) = self.group() else {
            return;
        };
        let separators: Vec<SeparatorEntry> = group.children.with_untracked(|children| {
            children
                .iter()
                .filter_map(|child| match child {
                    Child::Separator(separator) => Some(*separator),
                    Child::Panel(_) => None,
                })
                .collect()
        });
        let Some(index) = separators.iter().position(|entry| entry.key == separator) else {
            return;
        };
        let next = match (back, index) {
            (true, 0) => separators.len() - 1,
            (true, _) => index - 1,
            (false, _) => (index + 1) % separators.len(),
        };
        if let Some(element) = separators[next].node.get_untracked() {
            let options = FocusOptions::new();
            options.set_prevent_scroll(true);
            let _ = element.focus_with_options(&options);
        }
    }

    fn double_click(self, separator: SeparatorEntry, event: &ev::MouseEvent) {
        if event.default_prevented() || separator.disable_double_click {
            return;
        }
        let (Some(group), Some(pivots)) = (self.group(), self.usable(separator.key)) else {
            return;
        };
        let panels = group.children.with_untracked(|children| panels(children));
        let reset = pivots.into_iter().find_map(|index| {
            panels[index]
                .default_size
                .map(|size| (panels[index].key, size))
        });
        if let Some((key, size)) = reset {
            self.resize(key, size);
            event.prevent_default();
        }
    }

    fn panel(self, key: u64) -> Option<(Group, Vec<PanelEntry>, Measured, usize)> {
        let group = self.group()?;
        let (panels, measured) = Self::current(&group)?;
        let index = panels.iter().position(|panel| panel.key == key)?;
        Some((group, panels, measured, index))
    }

    fn set_panel_size(self, key: u64, size: f64) {
        let Some((_, panels, measured, index)) = self.panel(key) else {
            return;
        };
        let layout = &measured.layout;
        let constraints = &measured.constraints;
        if size == layout[index] || panels.len() < 2 {
            return;
        }
        let first = index == 0;
        let last = index == panels.len() - 1;
        // with every panel before it collapsed the last keeps what's left, rather than the first taking it
        let collapsed_before = last
            && size < layout[index]
            && (first
                || constraints[..index]
                    .iter()
                    .zip(layout)
                    .all(|(constraints, size)| {
                        constraints.collapsible && equal(constraints.collapsed_size, *size)
                    }));
        let next = if collapsed_before {
            let taken: f64 = layout[..index].iter().sum();
            let mut next = layout.clone();
            next[index] = fixed(100.0 - taken);
            next
        } else if last {
            adjust(
                layout[index] - size,
                layout,
                constraints,
                [index - 1, index],
                layout,
                Trigger::Api,
            )
        } else {
            adjust(
                size - layout[index],
                layout,
                constraints,
                [index, index + 1],
                layout,
                Trigger::Api,
            )
        };
        let next = validate_layout(&next, constraints);
        if !same(&next, layout) {
            self.commit(next, false);
        }
    }

    fn resize(self, key: u64, size: Length) {
        let Some((_, panels, measured, index)) = self.panel(key) else {
            return;
        };
        let element = panels[index].node.get_untracked();
        let pixels = size.pixels(measured.size, element.as_deref());
        self.set_panel_size(key, fixed((pixels / measured.size) * 100.0));
    }

    fn collapse(self, key: u64) {
        let Some((_, panels, measured, index)) = self.panel(key) else {
            return;
        };
        let constraints = measured.constraints[index];
        let size = measured.layout[index];
        if constraints.collapsible && size != constraints.collapsed_size {
            panels[index].expand_to.set_value(Some(size));
            self.set_panel_size(key, constraints.collapsed_size);
        }
    }

    fn expand(self, key: u64) {
        let Some((_, panels, measured, index)) = self.panel(key) else {
            return;
        };
        let constraints = measured.constraints[index];
        if constraints.collapsible && measured.layout[index] == constraints.collapsed_size {
            let size = panels[index]
                .expand_to
                .get_value()
                .unwrap_or(constraints.min_size);
            // a panel without a minimum still opens
            self.set_panel_size(key, if size == 0.0 { 1.0 } else { size });
        }
    }

    fn panel_size(self, key: u64) -> PanelSize {
        let Some(group) = self.group() else {
            return PanelSize::default();
        };
        group.measured.track();
        group.children.track();
        let Some((group, panels, measured, index)) = self.panel(key) else {
            return PanelSize::default();
        };
        PanelSize {
            as_percentage: measured.layout[index],
            in_pixels: Self::measure(&panels[index..=index], Self::horizontal(&group)),
        }
    }

    // as react-resizable-panels reports it to on_resize, from the page rather than the layout
    fn observed(self, key: u64) -> Option<PanelSize> {
        let group = self.group()?;
        let panels = group.children.with_untracked(|children| panels(children));
        let panel = panels.iter().find(|panel| panel.key == key)?;
        let horizontal = Self::horizontal(&group);
        let size = Self::measure(&panels, horizontal);
        let in_pixels = Self::measure(std::slice::from_ref(panel), horizontal);
        Some(PanelSize {
            as_percentage: fixed((in_pixels / size) * 100.0),
            in_pixels,
        })
    }

    fn is_collapsed(self, key: u64) -> bool {
        let Some(group) = self.group() else {
            return false;
        };
        group.measured.track();
        group.children.track();
        self.panel(key).is_some_and(|(_, _, measured, index)| {
            let constraints = measured.constraints[index];
            constraints.collapsible && equal(constraints.collapsed_size, measured.layout[index])
        })
    }

    fn set_layout(self, layout: &Layout) -> Layout {
        let Some(group) = self.group() else {
            return Layout::default();
        };
        let Some((panels, measured)) = Self::current(&group) else {
            return untrack(|| self.layout());
        };
        let Some(next) = ordered(layout, &panels) else {
            return untrack(|| self.layout());
        };
        let next = validate_layout(&next, &measured.constraints);
        if !same(&next, &measured.layout) {
            self.commit(next.clone(), false);
        }
        Self::named(&panels, &next)
    }
}

fn ids(panels: &[PanelEntry]) -> String {
    panels
        .iter()
        .map(|panel| panel.id.get())
        .collect::<Vec<_>>()
        .join(",")
}

fn ordered(layout: &Layout, panels: &[PanelEntry]) -> Option<Vec<f64>> {
    if layout.len() != panels.len() {
        return None;
    }
    panels
        .iter()
        .map(|panel| layout.get(&panel.id.get()))
        .collect()
}

fn number(value: f64) -> String {
    (value + 0.0).to_string()
}

fn same(a: &[f64], b: &[f64]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equal(*a, *b))
}

thread_local! {
    static CURSOR: RefCell<Option<(CssStyleSheet, Option<&'static str>)>> = const { RefCell::new(None) };
}

// the whole page takes the cursor, so it stays while a drag passes over other elements
fn show_cursor(cursor: Option<&'static str>) {
    CURSOR.with(|held| {
        let mut held = held.borrow_mut();
        if held.is_none() {
            if cursor.is_none() {
                return;
            }
            let Ok(sheet) = CssStyleSheet::new() else {
                return;
            };
            let document = document();
            let sheets = web_sys::js_sys::Array::from(&document.adopted_style_sheets());
            sheets.push(&sheet);
            document.set_adopted_style_sheets(&sheets);
            *held = Some((sheet, None));
        }
        let Some((sheet, shown)) = held.as_mut() else {
            return;
        };
        if *shown == cursor {
            return;
        }
        *shown = cursor;
        let rule = cursor.map_or_else(String::new, |cursor| {
            format!("*, *:hover {{cursor: {cursor} !important; }}")
        });
        let _ = sheet.replace_sync(&rule);
    });
}

fn cursor(orientation: Orientation, flags: u8) -> &'static str {
    // react-resizable-panels keeps safari to the cursors every browser draws
    let advanced = window()
        .navigator()
        .user_agent()
        .is_ok_and(|agent| agent.contains("Chrome") || agent.contains("Firefox"));
    if advanced && flags != 0 {
        let has = |flag: u8| flags & flag != 0;
        let vertical = |min: &'static str, max: &'static str, neither: &'static str| {
            if has(VERTICAL_MIN) {
                min
            } else if has(VERTICAL_MAX) {
                max
            } else {
                neither
            }
        };
        if has(HORIZONTAL_MIN) {
            return vertical("se-resize", "ne-resize", "e-resize");
        }
        if has(HORIZONTAL_MAX) {
            return vertical("sw-resize", "nw-resize", "w-resize");
        }
        if has(VERTICAL_MIN) {
            return "s-resize";
        }
        if has(VERTICAL_MAX) {
            return "n-resize";
        }
    }
    match (orientation, advanced) {
        (Orientation::Horizontal, true) => "ew-resize",
        (Orientation::Horizontal, false) => "col-resize",
        (Orientation::Vertical, true) => "ns-resize",
        (Orientation::Vertical, false) => "row-resize",
    }
}

// javascript's toFixed(3), which breaks an exact tie upwards where rust's formatting picks the even digit
fn fixed(value: f64) -> f64 {
    let sixteenths = value.abs() * 16.0;
    let rounded = if sixteenths.fract() == 0.0 && sixteenths % 2.0 == 1.0 {
        ((value.abs() * 1000.0).floor() + 1.0) / 1000.0 * value.signum()
    } else {
        format!("{value:.3}").parse().unwrap_or(value)
    };
    rounded + 0.0
}

fn equal_within(actual: f64, expected: f64, delta: f64) -> bool {
    (fixed(actual) - fixed(expected)).abs() <= delta
}

fn equal(actual: f64, expected: f64) -> bool {
    equal_within(actual, expected, 0.0)
}

fn compare(actual: f64, expected: f64) -> Ordering {
    if equal(actual, expected) {
        Ordering::Equal
    } else if actual > expected {
        Ordering::Greater
    } else {
        Ordering::Less
    }
}

fn validate_size(constraints: &Constraints, previous: f64, size: f64, unlock: bool) -> f64 {
    if constraints.disabled && !unlock {
        return previous;
    }
    let mut size = size;
    if compare(size, constraints.min_size) == Ordering::Less {
        let halfway = f64::midpoint(constraints.collapsed_size, constraints.min_size);
        size = if constraints.collapsible && compare(size, halfway) == Ordering::Less {
            constraints.collapsed_size
        } else {
            constraints.min_size
        };
    }
    fixed(constraints.max_size.min(size))
}

fn validate_layout(layout: &[f64], constraints: &[Constraints]) -> Vec<f64> {
    if layout.len() != constraints.len() {
        return default_layout(constraints);
    }
    let mut next = layout.to_vec();
    let total: f64 = next.iter().sum();
    if !equal(total, 100.0) && !next.is_empty() {
        for size in &mut next {
            *size *= 100.0 / total;
        }
    }
    let mut remaining = 0.0;
    for (index, constraints) in constraints.iter().enumerate() {
        let wanted = next[index];
        let safe = validate_size(constraints, layout[index], wanted, true);
        if wanted != safe {
            remaining += wanted - safe;
            next[index] = safe;
        }
    }
    if !equal(remaining, 0.0) {
        for (index, constraints) in constraints.iter().enumerate() {
            let size = next[index];
            let safe = validate_size(constraints, size, size + remaining, true);
            if size != safe {
                remaining -= safe - size;
                next[index] = safe;
                if equal(remaining, 0.0) {
                    break;
                }
            }
        }
    }
    next
}

fn default_layout(constraints: &[Constraints]) -> Vec<f64> {
    let mut total = 0.0;
    let mut open = 0.0;
    let stated: Vec<Option<f64>> = constraints
        .iter()
        .map(|constraints| {
            let size = constraints.default_size.map(fixed);
            match size {
                Some(size) => total += size,
                None => open += 1.0,
            }
            size
        })
        .collect();
    let rest = if open > 0.0 {
        fixed((100.0 - total) / open)
    } else {
        0.0
    };
    stated
        .into_iter()
        .map(|size| size.unwrap_or(rest))
        .collect()
}

fn preserve_fixed(
    constraints: &[Constraints],
    size: f64,
    previous_size: f64,
    layout: &[f64],
) -> Vec<f64> {
    if previous_size <= 0.0 || size <= 0.0 || previous_size == size {
        return layout.to_vec();
    }
    let mut next = layout.to_vec();
    let mut pinned = 0.0;
    let mut flexible = 0.0;
    let mut flexible_total = 0.0;
    for (index, constraints) in constraints.iter().enumerate() {
        match constraints.behavior {
            GroupResizeBehavior::PreservePixelSize => {
                next[index] = fixed(((layout[index] / 100.0) * previous_size / size) * 100.0);
                pinned += next[index];
            }
            GroupResizeBehavior::PreserveRelativeSize => {
                flexible += 1.0;
                flexible_total += layout[index];
            }
        }
    }
    let any_pinned = constraints
        .iter()
        .any(|constraints| constraints.behavior == GroupResizeBehavior::PreservePixelSize);
    if !any_pinned || flexible == 0.0 {
        return layout.to_vec();
    }
    let remaining = 100.0 - pinned;
    for (index, constraints) in constraints.iter().enumerate() {
        if constraints.behavior == GroupResizeBehavior::PreserveRelativeSize {
            next[index] = if flexible_total > 0.0 {
                fixed((layout[index] / flexible_total) * remaining)
            } else {
                fixed(remaining / flexible)
            };
        }
    }
    next
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Trigger {
    Keyboard,
    Pointer,
    Api,
}

// react-resizable-panels' adjustLayoutByDelta, a positive delta growing the panel before the separator
fn adjust(
    delta: f64,
    initial: &[f64],
    constraints: &[Constraints],
    pivots: [usize; 2],
    previous: &[f64],
    trigger: Trigger,
) -> Vec<f64> {
    if equal(delta, 0.0) {
        return initial.to_vec();
    }
    let count = constraints.len();
    let unlock = trigger == Trigger::Api;
    let [first, second] = pivots;
    let mut delta = delta;
    let mut next = initial.to_vec();
    let mut applied: f64 = 0.0;

    if trigger == Trigger::Keyboard {
        let index = if delta < 0.0 { second } else { first };
        let panel = constraints[index];
        if panel.collapsible && equal(initial[index], panel.collapsed_size) {
            let local = panel.min_size - initial[index];
            if compare(local, delta.abs()) == Ordering::Greater {
                delta = if delta < 0.0 { 0.0 - local } else { local };
            }
        }
        let index = if delta < 0.0 { first } else { second };
        let panel = constraints[index];
        if panel.collapsible && equal(initial[index], panel.min_size) {
            let local = initial[index] - panel.collapsed_size;
            if compare(local, delta.abs()) == Ordering::Greater {
                delta = if delta < 0.0 { 0.0 - local } else { local };
            }
        }
    } else {
        let index = if delta < 0.0 { second } else { first };
        let panel = constraints[index];
        let size = initial[index];
        if panel.collapsible && compare(size, panel.min_size) == Ordering::Less {
            let gap = panel.min_size - panel.collapsed_size;
            if delta > 0.0 {
                if compare(size + delta, panel.min_size) == Ordering::Less {
                    delta = if compare(delta, gap / 2.0) == Ordering::Greater {
                        gap
                    } else {
                        0.0
                    };
                }
            } else if compare(size - delta, panel.min_size) == Ordering::Less {
                delta = if compare(100.0 + delta, 100.0 - gap / 2.0) == Ordering::Greater {
                    0.0
                } else {
                    -gap
                };
            }
        }
    }

    {
        let forward = delta < 0.0;
        let mut index = if forward { second } else { first };
        let mut available = 0.0;
        loop {
            let size = initial[index];
            available += validate_size(&constraints[index], size, 100.0, unlock) - size;
            if forward {
                index += 1;
                if index >= count {
                    break;
                }
            } else if index == 0 {
                break;
            } else {
                index -= 1;
            }
        }
        let bounded = delta.abs().min(available.abs());
        delta = if delta < 0.0 { 0.0 - bounded } else { bounded };
    }

    {
        let backward = delta < 0.0;
        let mut index = if backward { first } else { second };
        loop {
            let remaining = delta.abs() - applied.abs();
            let size = initial[index];
            let safe = validate_size(&constraints[index], size, size - remaining, unlock);
            if !equal(size, safe) {
                applied += size - safe;
                next[index] = safe;
                if fixed(applied) >= fixed(delta.abs()) {
                    break;
                }
            }
            if backward {
                if index == 0 {
                    break;
                }
                index -= 1;
            } else {
                index += 1;
                if index >= count {
                    break;
                }
            }
        }
    }

    if previous == next.as_slice() {
        return previous.to_vec();
    }

    {
        let pivot = if delta < 0.0 { second } else { first };
        let size = initial[pivot];
        let wanted = size + applied;
        let safe = validate_size(&constraints[pivot], size, wanted, unlock);
        next[pivot] = safe;
        if !equal(safe, wanted) {
            let mut remaining = wanted - safe;
            let backward = delta > 0.0;
            let mut index = pivot;
            loop {
                let size = next[index];
                let safe = validate_size(&constraints[index], size, size + remaining, unlock);
                if !equal(size, safe) {
                    remaining -= safe - size;
                    next[index] = safe;
                }
                if equal(remaining, 0.0) {
                    break;
                }
                if backward {
                    if index == 0 {
                        break;
                    }
                    index -= 1;
                } else {
                    index += 1;
                    if index >= count {
                        break;
                    }
                }
            }
        }
    }

    let total: f64 = next.iter().sum();
    if !equal_within(total, 100.0, 0.1) {
        return previous.to_vec();
    }
    next
}

fn aria_values(layout: &[f64], constraints: &[Constraints], index: usize) -> (f64, f64) {
    let panel = constraints[index];
    let size = layout[index];
    let at = |delta: f64| {
        let adjusted = adjust(
            delta,
            layout,
            constraints,
            [index, index + 1],
            layout,
            Trigger::Pointer,
        );
        validate_layout(&adjusted, constraints)[index]
    };
    let min = if panel.collapsible {
        panel.collapsed_size
    } else {
        panel.min_size
    };
    (at(min - size), at(panel.max_size - size))
}

#[derive(Clone, Copy)]
pub struct GroupRef(RwSignal<Option<Resizable>>);

impl fmt::Debug for GroupRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GroupRef").finish_non_exhaustive()
    }
}

impl Default for GroupRef {
    fn default() -> Self {
        Self::new()
    }
}

impl GroupRef {
    pub fn new() -> Self {
        Self(RwSignal::new(None))
    }

    // empty until the group is measured
    pub fn layout(self) -> Layout {
        self.0.get().map(Resizable::layout).unwrap_or_default()
    }

    pub fn set_layout(self, layout: &Layout) -> Layout {
        self.0
            .get_untracked()
            .map(|resizable| resizable.set_layout(layout))
            .unwrap_or_default()
    }
}

#[derive(Clone, Copy)]
pub struct PanelRef(RwSignal<Option<(Resizable, u64)>>);

impl fmt::Debug for PanelRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PanelRef").finish_non_exhaustive()
    }
}

impl Default for PanelRef {
    fn default() -> Self {
        Self::new()
    }
}

impl PanelRef {
    pub fn new() -> Self {
        Self(RwSignal::new(None))
    }

    fn with(self, run: impl FnOnce(Resizable, u64)) {
        if let Some((resizable, key)) = self.0.get_untracked() {
            run(resizable, key);
        }
    }

    pub fn collapse(self) {
        self.with(Resizable::collapse);
    }

    pub fn expand(self) {
        self.with(Resizable::expand);
    }

    pub fn resize(self, size: impl Into<Length>) {
        let size = size.into();
        self.with(|resizable, key| resizable.resize(key, size));
    }

    pub fn size(self) -> PanelSize {
        self.0
            .get()
            .map(|(resizable, key)| resizable.panel_size(key))
            .unwrap_or_default()
    }

    pub fn is_collapsed(self) -> bool {
        self.0
            .get()
            .is_some_and(|(resizable, key)| resizable.is_collapsed(key))
    }
}

fn touch_action(orientation: Signal<Orientation>) -> impl Fn() -> &'static str + Copy {
    move || match orientation.get() {
        Orientation::Horizontal => "pan-y",
        Orientation::Vertical => "pan-x",
    }
}

#[component(element = "div")]
pub fn ResizableGroup(
    #[prop(into, default = Orientation::Horizontal.into())] orientation: Signal<Orientation>,
    #[prop(optional_no_strip)] default_layout: Option<Layout>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] disable_cursor: Signal<bool>,
    #[prop(optional_no_strip, into)] on_layout_change: Option<Callback<Layout>>,
    #[prop(optional_no_strip, into)] on_layout_changed: Option<
        Callback<(Layout, LayoutChangedMeta)>,
    >,
    #[prop(optional_no_strip)] group_ref: Option<GroupRef>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let node = NodeRef::new();
    let resizable = Resizable(StoredValue::new(Group {
        orientation,
        disabled,
        disable_cursor,
        node,
        children: RwSignal::new(Vec::new()),
        keys: StoredValue::new(0),
        measured: RwSignal::new(None),
        default_layout: StoredValue::new(default_layout),
        layouts: StoredValue::new(HashMap::new()),
        expanded: StoredValue::new(HashMap::new()),
        drag: StoredValue::new(None),
        active: RwSignal::new(None),
        reported: StoredValue::new(Reported::default()),
        on_layout_change,
        on_layout_changed,
    }));
    if let Some(group_ref) = group_ref {
        group_ref.0.set(Some(resizable));
        on_cleanup(move || {
            group_ref.0.try_set(None);
        });
    }

    Effect::new(move |_| {
        if node.get().is_none() {
            return;
        }
        orientation.track();
        if let Some(group) = resizable.group() {
            group.children.track();
        }
        untrack(move || resizable.mount());
    });
    Effect::new(move |_| {
        if let Some(element) = node.get() {
            observe_resize(&element, move || resizable.resized());
        }
    });

    view! {
        <div
            {..attrs}
            node_ref=node
            data-group=""
            style:height="100%"
            style:width="100%"
            style:overflow="hidden"
            style:display="flex"
            style:flex-direction=move || match orientation.get() {
                Orientation::Horizontal => "row",
                Orientation::Vertical => "column",
            }
            style:flex-wrap="nowrap"
            style:touch-action=touch_action(orientation)
        >
            <Provider value=resizable>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "div")]
pub fn ResizablePanel(
    #[prop(optional_no_strip, into)] id: Option<String>,
    #[prop(optional_no_strip, into)] default_size: Option<Length>,
    #[prop(optional_no_strip, into)] min_size: Option<Length>,
    #[prop(optional_no_strip, into)] max_size: Option<Length>,
    #[prop(optional_no_strip, into)] collapsed_size: Option<Length>,
    #[prop(optional)] collapsible: bool,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] group_resize_behavior: GroupResizeBehavior,
    #[prop(optional_no_strip, into)] on_resize: Option<Callback<(PanelSize, Option<PanelSize>)>>,
    #[prop(optional_no_strip)] panel_ref: Option<PanelRef>,
    #[prop(optional_no_strip, into)] class: Option<Signal<String>>,
    #[prop(optional_no_strip, into)] style: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let resizable = use_resizable("ResizablePanel");
    let node = NodeRef::new();
    let id = ElementId::new(node, id.unwrap_or_else(use_id));
    let expand_to = StoredValue::new(None);
    let key = resizable.register(|key| {
        Child::Panel(PanelEntry {
            key,
            id,
            node,
            default_size,
            min_size,
            max_size,
            collapsed_size,
            collapsible,
            disabled,
            behavior: group_resize_behavior,
            expand_to,
        })
    });
    on_cleanup(move || resizable.unregister(key));
    if let Some(panel_ref) = panel_ref {
        panel_ref.0.set(Some((resizable, key)));
        on_cleanup(move || {
            panel_ref.0.try_set(None);
        });
    }

    if let Some(on_resize) = on_resize {
        let previous = StoredValue::new(None);
        Effect::new(move |_| {
            if let Some(element) = node.get() {
                observe_resize(&element, move || {
                    let Some(size) = resizable.observed(key) else {
                        return;
                    };
                    let before = previous
                        .try_update_value(|previous| previous.replace(size))
                        .flatten();
                    on_resize.run((size, before));
                });
            }
        });
    }

    let flex = Memo::new(move |_| resizable.flex(key));
    let orientation = resizable
        .group()
        .map_or_else(|| Orientation::Horizontal.into(), |group| group.orientation);
    let dragging = move || {
        resizable
            .group()
            .and_then(|group| group.active.get())
            .map(|_| "none")
    };
    let style = style.map(leptos::tachys::html::style::style);
    let style = match style {
        Some(style) => Either::Left(style),
        None => Either::Right(()),
    };

    view! {
        <div
            {..attrs}
            node_ref=node
            id=id.get()
            data-panel=""
            data-disabled=move || disabled.get().then_some("true")
            style:min-height="0"
            style:max-height="100%"
            style:height="auto"
            style:min-width="0"
            style:max-width="100%"
            style:width="auto"
            style:border="none"
            style:border-width="0"
            style:padding="0"
            style:margin="0"
            style:display="flex"
            style:flex-basis=move || flex.with(|flex| flex.basis.clone())
            style:flex-shrink="1"
            style:overflow="visible"
            style:flex-grow=move || flex.with(|flex| flex.grow.clone())
            style:pointer-events=dragging
        >
            <div
                {..style}
                class=class
                style:max-height="100%"
                style:max-width="100%"
                style:flex-grow="1"
                style:overflow="auto"
                style:touch-action=touch_action(orientation)
            >
                {children.map(|children| children())}
            </div>
        </div>
    }
}

#[component(element = "div", skip(role, tabindex))]
pub fn ResizableSeparator(
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] disable_double_click: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let resizable = use_resizable("ResizableSeparator");
    let node = NodeRef::new();
    let hovered = RwSignal::new(false);
    let focused = RwSignal::new(false);
    let mut entry = SeparatorEntry {
        key: 0,
        node,
        disabled,
        disable_double_click,
        hovered,
    };
    entry.key = resizable.register(|key| Child::Separator(SeparatorEntry { key, ..entry }));
    let key = entry.key;
    on_cleanup(move || resizable.unregister(key));

    let aria = Memo::new(move |_| resizable.aria(key));
    let group = resizable.group();
    let orientation =
        group.map_or_else(|| Orientation::Horizontal.into(), |group| group.orientation);
    let disable_cursor = group.map_or_else(|| false.into(), |group| group.disable_cursor);
    let state = move || {
        let active = resizable.group().and_then(|group| group.active.get()) == Some(key);
        if disabled.get() {
            SeparatorState::Disabled
        } else if active {
            SeparatorState::Active
        } else if focused.get() {
            SeparatorState::Focus
        } else if hovered.get() {
            SeparatorState::Hover
        } else {
            SeparatorState::Inactive
        }
    };

    view! {
        <div
            {..attrs}
            node_ref=node
            role="separator"
            tabindex=move || (!disabled.get()).then_some("0")
            aria-controls=move || aria.with(|aria| aria.as_ref().map(|aria| aria.controls.clone()))
            aria-disabled=move || disabled.get().then_some("true")
            aria-orientation=move || match orientation.get() {
                Orientation::Horizontal => "vertical",
                Orientation::Vertical => "horizontal",
            }
            aria-valuemax=move || aria.with(|aria| aria.as_ref().map(|aria| number(aria.max)))
            aria-valuemin=move || aria.with(|aria| aria.as_ref().map(|aria| number(aria.min)))
            aria-valuenow=move || aria.with(|aria| aria.as_ref().map(|aria| number(aria.now)))
            data-separator=move || state().as_str()
            style:flex-basis="auto"
            style:cursor=move || (disabled.get() && !disable_cursor.get()).then_some("not-allowed")
            style:flex-grow="0"
            style:flex-shrink="0"
            style:touch-action="none"
            on:pointerdown=move |event| resizable.pointer_down(entry, &event)
            on:pointermove=move |event| resizable.pointer_move(&event)
            on:pointerup=move |_| resizable.end()
            on:pointercancel=move |_| resizable.end()
            on:lostpointercapture=move |_| resizable.end()
            on:contextmenu=move |event: ev::MouseEvent| {
                if !event.default_prevented() {
                    resizable.end();
                }
            }
            on:pointerenter=move |_| resizable.enter(entry)
            on:pointerleave=move |_| resizable.leave(entry)
            on:dblclick=move |event| resizable.double_click(entry, &event)
            on:keydown=move |event| resizable.key(entry, &event)
            on:focus=move |_| focused.set(true)
            on:blur=move |_| focused.set(false)
        >
            {children.map(|children| children())}
        </div>
    }
}
