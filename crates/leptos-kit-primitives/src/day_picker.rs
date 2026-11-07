use crate::date::{CalendarDate, DateRange, Weekday};
use crate::locale::{Labels, Locale};
use crate::{Direction, Keyword};
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_kit_macros::component;
use std::fmt;
use std::sync::Arc;
use web_sys::HtmlElement;

const STATUS_STYLE: &str = "border: 0; clip: rect(0 0 0 0); height: 1px; margin: -1px; overflow: hidden; \
    padding: 0; position: absolute; width: 1px; white-space: nowrap; word-wrap: normal";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Keyword)]
pub enum DayPickerMode {
    Single,
    Multiple,
    Range,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Keyword)]
pub enum CaptionLayout {
    #[default]
    Label,
    Dropdown,
    DropdownMonths,
    DropdownYears,
}

impl CaptionLayout {
    fn months_dropdown(self) -> bool {
        matches!(self, Self::Dropdown | Self::DropdownMonths)
    }

    fn years_dropdown(self) -> bool {
        matches!(self, Self::Dropdown | Self::DropdownYears)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Keyword)]
pub enum NavLayout {
    Around,
    After,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChevronOrientation {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone)]
pub enum Matcher {
    Bool(bool),
    Date(CalendarDate),
    Dates(Vec<CalendarDate>),
    Range(DateRange),
    DaysOfWeek(Vec<Weekday>),
    Before(CalendarDate),
    After(CalendarDate),
    Interval {
        before: CalendarDate,
        after: CalendarDate,
    },
    Fn(Arc<dyn Fn(CalendarDate) -> bool + Send + Sync>),
    Any(Vec<Matcher>),
}

impl fmt::Debug for Matcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(value) => f.debug_tuple("Bool").field(value).finish(),
            Self::Date(date) => f.debug_tuple("Date").field(date).finish(),
            Self::Dates(dates) => f.debug_tuple("Dates").field(dates).finish(),
            Self::Range(range) => f.debug_tuple("Range").field(range).finish(),
            Self::DaysOfWeek(days) => f.debug_tuple("DaysOfWeek").field(days).finish(),
            Self::Before(date) => f.debug_tuple("Before").field(date).finish(),
            Self::After(date) => f.debug_tuple("After").field(date).finish(),
            Self::Interval { before, after } => f
                .debug_struct("Interval")
                .field("before", before)
                .field("after", after)
                .finish(),
            Self::Fn(_) => f.write_str("Fn(..)"),
            Self::Any(matchers) => f.debug_tuple("Any").field(matchers).finish(),
        }
    }
}

impl From<bool> for Matcher {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<CalendarDate> for Matcher {
    fn from(date: CalendarDate) -> Self {
        Self::Date(date)
    }
}

impl From<Vec<CalendarDate>> for Matcher {
    fn from(dates: Vec<CalendarDate>) -> Self {
        Self::Dates(dates)
    }
}

impl From<DateRange> for Matcher {
    fn from(range: DateRange) -> Self {
        Self::Range(range)
    }
}

impl From<Weekday> for Matcher {
    fn from(weekday: Weekday) -> Self {
        Self::DaysOfWeek(vec![weekday])
    }
}

impl From<Vec<Weekday>> for Matcher {
    fn from(weekdays: Vec<Weekday>) -> Self {
        Self::DaysOfWeek(weekdays)
    }
}

impl From<Vec<Matcher>> for Matcher {
    fn from(matchers: Vec<Matcher>) -> Self {
        Self::Any(matchers)
    }
}

impl<F: Fn(CalendarDate) -> bool + Send + Sync + 'static> From<F> for Matcher {
    fn from(matches: F) -> Self {
        Self::Fn(Arc::new(matches))
    }
}

impl Matcher {
    pub fn matches(&self, date: CalendarDate) -> bool {
        match self {
            Self::Bool(value) => *value,
            Self::Date(other) => *other == date,
            Self::Dates(dates) => dates.contains(&date),
            Self::Range(range) => includes(*range, date, false),
            Self::DaysOfWeek(weekdays) => weekdays.contains(&date.weekday()),
            Self::Before(before) => before.days_since(date) > 0,
            Self::After(after) => date.days_since(*after) > 0,
            Self::Interval { before, after } => {
                let is_before = before.days_since(date) > 0;
                let is_after = after.days_since(date) < 0;
                if before > after {
                    is_after && is_before
                } else {
                    is_before || is_after
                }
            }
            Self::Fn(matches) => matches(date),
            Self::Any(matchers) => matchers.iter().any(|matcher| matcher.matches(date)),
        }
    }

    fn overlaps(&self, from: CalendarDate, to: CalendarDate) -> bool {
        let range = DateRange::new(from, to);
        match self {
            Self::Bool(value) => *value,
            Self::Date(date) => includes(range, *date, false),
            Self::Dates(dates) => dates.iter().any(|date| includes(range, *date, false)),
            Self::Range(DateRange {
                from: Some(start),
                to: Some(end),
            }) => overlap(range, DateRange::new(*start, *end)),
            Self::Range(_) => false,
            Self::DaysOfWeek(weekdays) => (0..=to.days_since(from).min(6))
                .any(|offset| weekdays.contains(&from.add_days(offset).weekday())),
            Self::Interval { before, after } if before > after => overlap(
                range,
                DateRange::new(after.add_days(1), before.add_days(-1)),
            ),
            Self::Interval { .. } | Self::Before(_) | Self::After(_) => {
                self.matches(from) || self.matches(to)
            }
            Self::Fn(matches) => {
                (0..=to.days_since(from)).any(|offset| matches(from.add_days(offset)))
            }
            Self::Any(matchers) => matchers.iter().any(|matcher| matcher.overlaps(from, to)),
        }
    }
}

fn includes(range: DateRange, date: CalendarDate, exclude_ends: bool) -> bool {
    match (range.from, range.to) {
        (Some(from), Some(to)) => {
            let (from, to) = if to < from { (to, from) } else { (from, to) };
            let min = i32::from(exclude_ends);
            date.days_since(from) >= min && to.days_since(date) >= min
        }
        (_, Some(to)) => !exclude_ends && to == date,
        (Some(from), None) => !exclude_ends && from == date,
        (None, None) => false,
    }
}

fn overlap(left: DateRange, right: DateRange) -> bool {
    let ends = |range: DateRange| [range.from, range.to].into_iter().flatten();
    ends(right).any(|date| includes(left, date, false))
        || ends(left).any(|date| includes(right, date, false))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CalendarDay {
    pub date: CalendarDate,
    pub display_month: CalendarDate,
    pub outside: bool,
}

impl CalendarDay {
    fn new(date: CalendarDate, display_month: CalendarDate) -> Self {
        let display_month = display_month.start_of_month();
        Self {
            date,
            display_month,
            outside: date.start_of_month() != display_month,
        }
    }
}

// a range's flags are none unless a range is selected, as upstream leaves them out
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "react-day-picker's flags, each one a data attribute"
)]
pub struct Modifiers {
    pub focused: bool,
    pub disabled: bool,
    pub hidden: bool,
    pub outside: bool,
    pub today: bool,
    pub selected: bool,
    pub range_start: Option<bool>,
    pub range_end: Option<bool>,
    pub range_middle: Option<bool>,
    pub custom: Vec<&'static str>,
}

impl Modifiers {
    pub fn has(&self, name: &str) -> bool {
        self.custom.contains(&name)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClassNames {
    pub root: Option<String>,
    pub chevron: Option<String>,
    pub day: Option<String>,
    pub day_button: Option<String>,
    pub caption_label: Option<String>,
    pub dropdowns: Option<String>,
    pub dropdown: Option<String>,
    pub dropdown_root: Option<String>,
    pub footer: Option<String>,
    pub month_grid: Option<String>,
    pub month_caption: Option<String>,
    pub months_dropdown: Option<String>,
    pub month: Option<String>,
    pub months: Option<String>,
    pub nav: Option<String>,
    pub button_next: Option<String>,
    pub button_previous: Option<String>,
    pub week: Option<String>,
    pub weeks: Option<String>,
    pub weekday: Option<String>,
    pub weekdays: Option<String>,
    pub week_number: Option<String>,
    pub week_number_header: Option<String>,
    pub years_dropdown: Option<String>,
    pub disabled: Option<String>,
    pub hidden: Option<String>,
    pub outside: Option<String>,
    pub focused: Option<String>,
    pub today: Option<String>,
    pub range_end: Option<String>,
    pub range_middle: Option<String>,
    pub range_start: Option<String>,
    pub selected: Option<String>,
}

impl ClassNames {
    // the classes set here win over `base`'s, as a spread of `classNames` does upstream
    #[must_use]
    pub fn or(self, base: Self) -> Self {
        Self {
            root: self.root.or(base.root),
            chevron: self.chevron.or(base.chevron),
            day: self.day.or(base.day),
            day_button: self.day_button.or(base.day_button),
            caption_label: self.caption_label.or(base.caption_label),
            dropdowns: self.dropdowns.or(base.dropdowns),
            dropdown: self.dropdown.or(base.dropdown),
            dropdown_root: self.dropdown_root.or(base.dropdown_root),
            footer: self.footer.or(base.footer),
            month_grid: self.month_grid.or(base.month_grid),
            month_caption: self.month_caption.or(base.month_caption),
            months_dropdown: self.months_dropdown.or(base.months_dropdown),
            month: self.month.or(base.month),
            months: self.months.or(base.months),
            nav: self.nav.or(base.nav),
            button_next: self.button_next.or(base.button_next),
            button_previous: self.button_previous.or(base.button_previous),
            week: self.week.or(base.week),
            weeks: self.weeks.or(base.weeks),
            weekday: self.weekday.or(base.weekday),
            weekdays: self.weekdays.or(base.weekdays),
            week_number: self.week_number.or(base.week_number),
            week_number_header: self.week_number_header.or(base.week_number_header),
            years_dropdown: self.years_dropdown.or(base.years_dropdown),
            disabled: self.disabled.or(base.disabled),
            hidden: self.hidden.or(base.hidden),
            outside: self.outside.or(base.outside),
            focused: self.focused.or(base.focused),
            today: self.today.or(base.today),
            range_end: self.range_end.or(base.range_end),
            range_middle: self.range_middle.or(base.range_middle),
            range_start: self.range_start.or(base.range_start),
            selected: self.selected.or(base.selected),
        }
    }
}

type Pick = fn(&ClassNames) -> &Option<String>;

#[derive(Clone, Copy, Default)]
pub struct Formatters {
    pub format_caption: Option<Callback<CalendarDate, String>>,
    pub format_day: Option<Callback<CalendarDate, String>>,
    pub format_month_dropdown: Option<Callback<CalendarDate, String>>,
    pub format_year_dropdown: Option<Callback<CalendarDate, String>>,
    pub format_weekday_name: Option<Callback<Weekday, String>>,
    pub format_week_number: Option<Callback<i32, String>>,
    pub format_week_number_header: Option<Callback<(), String>>,
}

impl fmt::Debug for Formatters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Formatters").finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct ChevronProps {
    pub orientation: ChevronOrientation,
    pub class: String,
    pub disabled: Signal<bool>,
    pub size: u32,
}

#[derive(Clone, Debug)]
pub struct DayButtonProps {
    pub day: CalendarDay,
    pub modifiers: Signal<Modifiers>,
    pub class: String,
    pub children: String,
}

#[derive(Clone, Debug)]
pub struct WeekNumberProps {
    pub week_number: i32,
    pub class: String,
    pub label: String,
    pub children: String,
}

// only the parts shadcn replaces take a slot
#[derive(Clone, Copy, Default)]
pub struct DayPickerComponents {
    pub chevron: Option<Callback<ChevronProps, AnyView>>,
    pub day_button: Option<Callback<DayButtonProps, AnyView>>,
    pub week_number: Option<Callback<WeekNumberProps, AnyView>>,
}

impl fmt::Debug for DayPickerComponents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DayPickerComponents")
            .field("chevron", &self.chevron.is_some())
            .field("day_button", &self.day_button.is_some())
            .field("week_number", &self.week_number.is_some())
            .finish()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DayPickerDay {
    pub day: CalendarDay,
    pub modifiers: Signal<Modifiers>,
    pub locale: Signal<Locale>,
}

pub fn use_day_picker_day() -> DayPickerDay {
    expect_context()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Week {
    number: i32,
    days: Vec<CalendarDay>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Month {
    month: CalendarDate,
    weeks: Vec<Week>,
}

// `on_select` makes `selected` controlled, without it `selected` is only the first value
struct Choice<T: Send + Sync + 'static> {
    value: Signal<T>,
    owned: Option<RwSignal<T>>,
    on_select: Option<Callback<T>>,
}

impl<T: Send + Sync + 'static> Clone for Choice<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Send + Sync + 'static> Copy for Choice<T> {}

impl<T: Clone + Send + Sync + 'static> Choice<T> {
    fn new(selected: Option<Signal<T>>, on_select: Option<Callback<T>>, empty: T) -> Self {
        if on_select.is_some() {
            let value = selected.unwrap_or_else(|| Signal::stored(empty));
            return Self {
                value,
                owned: None,
                on_select,
            };
        }
        let owned = RwSignal::new(selected.map_or(empty, |selected| selected.get_untracked()));
        Self {
            value: owned.into(),
            owned: Some(owned),
            on_select,
        }
    }

    fn set(self, next: T) {
        if let Some(owned) = self.owned {
            owned.set(next.clone());
        }
        if let Some(on_select) = self.on_select {
            on_select.run(next);
        }
    }
}

#[derive(Clone, Copy)]
struct State {
    mode: Option<Signal<DayPickerMode>>,
    single: Choice<Option<CalendarDate>>,
    multiple: Choice<Vec<CalendarDate>>,
    range: Choice<Option<DateRange>>,
    required: Option<Signal<bool>>,
    min: Option<Signal<u32>>,
    max: Option<Signal<u32>>,
    exclude_disabled: Signal<bool>,
    reset_on_select: Signal<bool>,
    month: Option<Signal<CalendarDate>>,
    owned_month: RwSignal<CalendarDate>,
    default_month: Option<CalendarDate>,
    on_month_change: Option<Callback<CalendarDate>>,
    end_month: Option<Signal<CalendarDate>>,
    number_of_months: Signal<u32>,
    reverse_months: Signal<bool>,
    paged_navigation: Signal<bool>,
    disable_navigation: Signal<bool>,
    caption_layout: Signal<CaptionLayout>,
    reverse_years: Signal<bool>,
    show_outside_days: Signal<bool>,
    fixed_weeks: Signal<bool>,
    iso_week: Signal<bool>,
    week_starts_on: Option<Signal<Weekday>>,
    first_week_contains_date: Option<Signal<u8>>,
    disabled: Option<Signal<Matcher>>,
    hidden: Option<Signal<Matcher>>,
    modifiers: Option<Signal<Vec<(&'static str, Matcher)>>>,
    modifiers_class_names: Option<Signal<Vec<(&'static str, String)>>>,
    today: Signal<CalendarDate>,
    locale: Signal<Locale>,
    labels: Signal<Labels>,
    dir: Option<Signal<Direction>>,
    class_names: Option<Signal<ClassNames>>,
    formatters: Formatters,
    components: DayPickerComponents,
    focused: RwSignal<Option<CalendarDay>>,
    last_focused: RwSignal<Option<CalendarDay>>,
    nav: Memo<(Option<CalendarDate>, Option<CalendarDate>)>,
    months: Memo<Vec<Month>>,
    previous_month: Memo<Option<CalendarDate>>,
    next_month: Memo<Option<CalendarDate>>,
    focus_target: Memo<Option<CalendarDay>>,
    root: NodeRef<html::Div>,
}

// an 8-byte handle, so the closures of every day don't each copy the whole state
#[derive(Clone, Copy)]
struct DayPickerContext(StoredValue<State>);

impl DayPickerContext {
    fn state(self) -> Option<State> {
        self.0.try_get_value()
    }

    fn with<R>(self, read: impl FnOnce(State) -> R) -> Option<R> {
        self.0.try_with_value(|state| read(*state))
    }

    fn class(self, pick: Pick, default: &'static str) -> String {
        self.with(|this| this.class(pick, default))
            .unwrap_or_else(|| default.to_string())
    }
}

impl State {
    fn mode(self) -> Option<DayPickerMode> {
        self.mode.map(|mode| mode.get())
    }

    fn week_starts_on(self) -> Weekday {
        if self.iso_week.get() {
            return Weekday::Monday;
        }
        self.week_starts_on.map_or_else(
            || self.locale.with(Locale::first_weekday),
            |weekday| weekday.get(),
        )
    }

    fn week_number(self, date: CalendarDate) -> i32 {
        if self.iso_week.get() {
            return date.iso_week();
        }
        // icu4x has no cldr minDays, so the week holding january 1st is the first, as react-day-picker's default
        let first = self.first_week_contains_date.map_or(1, |first| first.get());
        date.week(self.week_starts_on(), first)
    }

    fn class(self, pick: Pick, default: &'static str) -> String {
        self.class_names
            .and_then(|names| names.with(|names| pick(names).clone()))
            .unwrap_or_else(|| default.to_string())
    }

    fn is_selected(self, date: CalendarDate) -> bool {
        match self.mode() {
            Some(DayPickerMode::Single) => self.single.value.get() == Some(date),
            Some(DayPickerMode::Multiple) => {
                self.multiple.value.with(|dates| dates.contains(&date))
            }
            Some(DayPickerMode::Range) => self
                .range
                .value
                .get()
                .is_some_and(|range| includes(range, date, false)),
            None => false,
        }
    }

    fn focusable(self, day: CalendarDay) -> bool {
        let (disabled, hidden) = self.disabled_hidden(day);
        !disabled && !hidden && !day.outside
    }

    fn disabled_hidden(self, day: CalendarDay) -> (bool, bool) {
        let date = day.date;
        let (start, end) = self.nav.get();
        let before_start = start.is_some_and(|start| date < start.start_of_month());
        let after_end = end.is_some_and(|end| date > end.end_of_month());
        let disabled = self
            .disabled
            .is_some_and(|disabled| disabled.with(|matcher| matcher.matches(date)));
        let hidden = self
            .hidden
            .is_some_and(|hidden| hidden.with(|matcher| matcher.matches(date)))
            || before_start
            || after_end
            || (!self.show_outside_days.get() && day.outside);
        (disabled, hidden)
    }

    fn modifiers(self, day: CalendarDay) -> Modifiers {
        let date = day.date;
        let (disabled, hidden) = self.disabled_hidden(day);
        let custom: Vec<&'static str> = self.modifiers.map_or_else(Vec::new, |modifiers| {
            modifiers.with(|modifiers| {
                modifiers
                    .iter()
                    .filter(|(_, matcher)| matcher.matches(date))
                    .map(|(name, _)| *name)
                    .collect()
            })
        });
        let selected = self.is_selected(date) || custom.contains(&"selected");
        let range = (self.mode() == Some(DayPickerMode::Range))
            .then(|| self.range.value.get())
            .flatten();
        let ends = |end: Option<CalendarDate>| {
            range.map(|range| range.from.is_some() && range.to.is_some() && end == Some(date))
        };
        Modifiers {
            focused: !hidden && self.focused.get() == Some(day),
            disabled,
            hidden,
            outside: day.outside,
            today: date == self.today.get(),
            selected,
            range_start: ends(range.and_then(|range| range.from)),
            range_end: ends(range.and_then(|range| range.to)),
            range_middle: range.map(|range| includes(range, date, true)),
            custom,
        }
    }

    fn day_class(self, modifiers: &Modifiers) -> String {
        let mut classes = vec![self.class(|names| &names.day, "rdp-day")];
        let own = |name: &str| {
            self.modifiers_class_names.and_then(|names| {
                names.with(|names| {
                    names
                        .iter()
                        .find(|(key, _)| *key == name)
                        .map(|(_, class)| class.clone())
                })
            })
        };
        let flags: [(bool, &str, Pick, &'static str); 5] = [
            (
                modifiers.focused,
                "focused",
                |names| &names.focused,
                "rdp-focused",
            ),
            (
                modifiers.disabled,
                "disabled",
                |names| &names.disabled,
                "rdp-disabled",
            ),
            (
                modifiers.hidden,
                "hidden",
                |names| &names.hidden,
                "rdp-hidden",
            ),
            (
                modifiers.outside,
                "outside",
                |names| &names.outside,
                "rdp-outside",
            ),
            (modifiers.today, "today", |names| &names.today, "rdp-today"),
        ];
        for (active, name, pick, default) in flags {
            if active {
                classes.push(own(name).unwrap_or_else(|| self.class(pick, default)));
            }
        }
        classes.extend(modifiers.custom.iter().filter_map(|name| own(name)));
        let states: [(bool, &str, Pick, &'static str); 4] = [
            (
                modifiers.selected,
                "selected",
                |names| &names.selected,
                "rdp-selected",
            ),
            (
                modifiers.range_start == Some(true),
                "range_start",
                |names| &names.range_start,
                "rdp-range_start",
            ),
            (
                modifiers.range_end == Some(true),
                "range_end",
                |names| &names.range_end,
                "rdp-range_end",
            ),
            (
                modifiers.range_middle == Some(true),
                "range_middle",
                |names| &names.range_middle,
                "rdp-range_middle",
            ),
        ];
        for (active, name, pick, default) in states {
            if active {
                classes.push(own(name).unwrap_or_else(|| self.class(pick, default)));
            }
        }
        classes.join(" ")
    }

    fn day_label(self, date: CalendarDate, modifiers: &Modifiers, button: bool) -> String {
        let mut label = self.locale.with(|locale| locale.long_date(date));
        if modifiers.today {
            label = self
                .labels
                .with(|labels| format!("{}, {label}", labels.today));
        }
        if button && modifiers.selected {
            label = self
                .labels
                .with(|labels| format!("{label}, {}", labels.selected));
        }
        label
    }

    fn format_caption(self, month: CalendarDate) -> String {
        self.formatters.format_caption.map_or_else(
            || self.locale.with(|locale| locale.month_year(month)),
            |format| format.run(month),
        )
    }

    fn format_month_dropdown(self, month: CalendarDate) -> String {
        self.formatters.format_month_dropdown.map_or_else(
            || self.locale.with(|locale| locale.month(month)),
            |format| format.run(month),
        )
    }

    fn format_year_dropdown(self, year: CalendarDate) -> String {
        self.formatters.format_year_dropdown.map_or_else(
            || self.locale.with(|locale| locale.year(year)),
            |format| format.run(year),
        )
    }

    fn format_day(self, date: CalendarDate) -> String {
        self.formatters.format_day.map_or_else(
            || self.locale.with(|locale| locale.day(date)),
            |format| format.run(date),
        )
    }

    fn format_weekday(self, weekday: Weekday) -> String {
        self.formatters.format_weekday_name.map_or_else(
            || self.locale.with(|locale| locale.weekday_short(weekday)),
            |format| format.run(weekday),
        )
    }

    fn format_week_number(self, number: i32) -> String {
        self.formatters.format_week_number.map_or_else(
            || {
                if number < 10 {
                    format!("0{number}")
                } else {
                    number.to_string()
                }
            },
            |format| format.run(number),
        )
    }

    fn first_month(self) -> CalendarDate {
        match self.month {
            Some(month) => self.initial_month(Some(month.get())),
            None => self.owned_month.get(),
        }
    }

    fn initial_month(self, month: Option<CalendarDate>) -> CalendarDate {
        let (start, end) = self.nav.get();
        let count = i32::try_from(self.number_of_months.get())
            .unwrap_or(1)
            .max(1);
        let mut initial = month
            .or(self.default_month)
            .unwrap_or_else(|| self.today.get());
        if let Some(end) = end
            && end.months_since(initial) < count - 1
        {
            initial = end.add_months(-(count - 1));
        }
        if let Some(start) = start
            && initial.months_since(start) < 0
        {
            initial = start;
        }
        initial.start_of_month()
    }

    fn go_to_month(self, date: CalendarDate) {
        if self.disable_navigation.get_untracked() {
            return;
        }
        let (start, end) = self.nav.get_untracked();
        let mut month = date.start_of_month();
        if let Some(start) = start
            && month < start.start_of_month()
        {
            month = start.start_of_month();
        }
        if let Some(end) = end
            && month > end.start_of_month()
        {
            month = end.start_of_month();
        }
        self.owned_month.set(month);
        if let Some(on_month_change) = self.on_month_change {
            on_month_change.run(month);
        }
    }

    fn in_calendar(self, day: CalendarDay) -> bool {
        self.months.with_untracked(|months| {
            months
                .iter()
                .flat_map(|month| &month.weeks)
                .any(|week| week.days.contains(&day))
        })
    }

    fn select(self, date: CalendarDate) {
        let required = self
            .required
            .is_some_and(|required| required.get_untracked());
        match self.mode.map(|mode| mode.get_untracked()) {
            Some(DayPickerMode::Single) => {
                let selected = self.single.value.get_untracked();
                let next = (required || selected != Some(date)).then_some(date);
                self.single.set(next);
            }
            Some(DayPickerMode::Multiple) => {
                let selected = self.multiple.value.get_untracked();
                let len = u32::try_from(selected.len()).unwrap_or(u32::MAX);
                let next = if selected.contains(&date) {
                    if self.min.map(|min| min.get_untracked()) == Some(len)
                        || (required && len == 1)
                    {
                        return;
                    }
                    selected
                        .into_iter()
                        .filter(|other| *other != date)
                        .collect()
                } else if self.max.map(|max| max.get_untracked()) == Some(len) {
                    vec![date]
                } else {
                    let mut next = selected;
                    next.push(date);
                    next
                };
                self.multiple.set(next);
            }
            Some(DayPickerMode::Range) => self.select_range(date, required),
            None => {}
        }
    }

    fn select_range(self, date: CalendarDate, required: bool) {
        let selected = self.range.value.get_untracked();
        let from = selected.and_then(|range| range.from);
        let to = selected.and_then(|range| range.to);
        let full = from.is_some() && to.is_some();
        let mut next = if self.reset_on_select.get_untracked() && (full || from.is_none()) {
            let single_day = full && from == to && from == Some(date);
            (required || !single_day).then_some(DateRange {
                from: Some(date),
                to: None,
            })
        } else {
            let limit = |limit: Option<Signal<u32>>| {
                limit.map_or(0, |limit| {
                    i32::try_from(limit.get_untracked()).unwrap_or(i32::MAX)
                })
            };
            add_to_range(date, selected, limit(self.min), limit(self.max), required)
        };
        if self.exclude_disabled.get_untracked()
            && let Some(disabled) = self.disabled
            && let Some(DateRange {
                from: Some(start),
                to: Some(end),
            }) = next
            && disabled.with_untracked(|matcher| matcher.overlaps(start, end))
        {
            next = Some(DateRange {
                from: Some(date),
                to: None,
            });
        }
        self.range.set(next);
    }

    fn next_focus(self, by: MoveBy, after: bool, from: CalendarDay) -> Option<CalendarDay> {
        let (start, end) = self.nav.get_untracked();
        let week_starts_on = untrack(|| self.week_starts_on());
        let step = if after { 1 } else { -1 };
        let mut date = from.date;
        for _ in 0..=365 {
            date = match by {
                MoveBy::Day => date.add_days(step),
                MoveBy::Week => date.add_days(7 * step),
                MoveBy::Month => date.add_months(step),
                MoveBy::Year => date.add_years(step),
                MoveBy::StartOfWeek => date.start_of_week(week_starts_on),
                MoveBy::EndOfWeek => date.end_of_week(week_starts_on),
            };
            if !after && let Some(start) = start {
                date = date.max(start.start_of_month());
            } else if after && let Some(end) = end {
                date = date.min(end.end_of_month());
            }
            let blocked = |matcher: Option<Signal<Matcher>>| {
                matcher
                    .is_some_and(|matcher| matcher.with_untracked(|matcher| matcher.matches(date)))
            };
            if !blocked(self.disabled) && !blocked(self.hidden) {
                return Some(CalendarDay::new(date, date));
            }
        }
        None
    }

    fn move_focus(self, by: MoveBy, after: bool) {
        let Some(focused) = self.focused.get_untracked() else {
            return;
        };
        let Some(next) = self.next_focus(by, after, focused) else {
            return;
        };
        if self.disable_navigation.get_untracked() && !self.in_calendar(next) {
            return;
        }
        if !self.in_calendar(next) {
            self.go_to_month(next.date);
        }
        self.focused.set(Some(next));
        self.focus_day(next);
    }

    fn focus_day(self, day: CalendarDay) {
        let root = self.root;
        request_animation_frame(move || {
            let Some(root) = root.get_untracked() else {
                return;
            };
            let selector = format!("td[data-day=\"{}\"]:not([data-month]) button", day.date);
            if let Ok(Some(button)) = root.query_selector(&selector)
                && let Ok(button) = button.dyn_into::<HtmlElement>()
            {
                let _ = button.focus();
            }
        });
    }

    fn key_down(self, event: &ev::KeyboardEvent) {
        let rtl = self
            .dir
            .is_some_and(|dir| dir.get_untracked() == Direction::Rtl);
        let shift = event.shift_key();
        let (by, after) = match event.key().as_str() {
            "ArrowLeft" => (if shift { MoveBy::Month } else { MoveBy::Day }, rtl),
            "ArrowRight" => (if shift { MoveBy::Month } else { MoveBy::Day }, !rtl),
            "ArrowDown" => (if shift { MoveBy::Year } else { MoveBy::Week }, true),
            "ArrowUp" => (if shift { MoveBy::Year } else { MoveBy::Week }, false),
            "PageUp" => (if shift { MoveBy::Year } else { MoveBy::Month }, false),
            "PageDown" => (if shift { MoveBy::Year } else { MoveBy::Month }, true),
            "Home" => (MoveBy::StartOfWeek, false),
            "End" => (MoveBy::EndOfWeek, true),
            _ => return,
        };
        event.prevent_default();
        event.stop_propagation();
        self.move_focus(by, after);
    }

    fn dropdown_change(self, month: CalendarDate, offset: i32, value: &str, years: bool) {
        let Ok(value) = value.parse::<i32>() else {
            return;
        };
        let start = month.start_of_month();
        let target = if years {
            start.with_year(value)
        } else {
            start.with_month(u8::try_from(value + 1).unwrap_or(1))
        };
        self.go_to_month(target.add_months(-offset));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MoveBy {
    Day,
    Week,
    Month,
    Year,
    StartOfWeek,
    EndOfWeek,
}

fn add_to_range(
    date: CalendarDate,
    range: Option<DateRange>,
    min: i32,
    max: i32,
    required: bool,
) -> Option<DateRange> {
    let from = range.and_then(|range| range.from);
    let to = range.and_then(|range| range.to);
    let open = |from: CalendarDate| DateRange {
        from: Some(from),
        to: (min == 0).then_some(date),
    };
    let next = match (from, to) {
        (None, _) => Some(open(date)),
        (Some(from), None) if from == date => {
            if min == 0 {
                Some(DateRange::new(from, date))
            } else if required {
                Some(DateRange {
                    from: Some(from),
                    to: None,
                })
            } else {
                None
            }
        }
        (Some(from), None) if date < from => Some(DateRange::new(date, from)),
        (Some(from), Some(to)) if from == date && to == date => {
            required.then_some(DateRange::new(from, to))
        }
        (Some(from), Some(_)) if from == date => Some(open(from)),
        (Some(_), Some(to)) if to == date => Some(open(date)),
        (Some(from), Some(to)) if date < from => Some(DateRange::new(date, to)),
        (Some(from), _) => Some(DateRange::new(from, date)),
    };
    if let Some(DateRange {
        from: Some(start),
        to: Some(end),
    }) = next
    {
        let days = end.days_since(start);
        if (max > 0 && days > max) || (min > 1 && days < min) {
            return Some(DateRange {
                from: Some(date),
                to: None,
            });
        }
    }
    next
}

fn layout(state: State, first: CalendarDate) -> Vec<Month> {
    let (_, nav_end) = state.nav.get();
    let count = i32::try_from(state.number_of_months.get())
        .unwrap_or(1)
        .max(1);
    let week_starts_on = state.week_starts_on();
    let fixed = state.fixed_weeks.get();
    let display: Vec<CalendarDate> = (0..count)
        .map(|index| first.add_months(index))
        .take_while(|month| nav_end.is_none_or(|end| *month <= end))
        .collect();
    let (Some(first), Some(last)) = (display.first().copied(), display.last().copied()) else {
        return Vec::new();
    };
    let start = first.start_of_week(week_starts_on);
    let mut end = last.end_of_month().end_of_week(week_starts_on);
    if let Some(limit) = state
        .end_month
        .map(|end| end.get().end_of_month().end_of_week(week_starts_on))
        && end > limit
    {
        end = limit;
    }
    let mut dates: Vec<CalendarDate> = (0..=end.days_since(start))
        .map(|offset| start.add_days(offset))
        .collect();
    let wanted = 42 * usize::try_from(last.months_since(first) + 1).unwrap_or(1);
    if fixed && dates.len() < wanted {
        let tail = dates.last().copied().unwrap_or(end);
        dates.extend(
            (1..=wanted - dates.len())
                .map(|offset| tail.add_days(i32::try_from(offset).unwrap_or(0))),
        );
    }
    let mut months: Vec<Month> = display
        .into_iter()
        .map(|month| {
            let first_of_grid = month.start_of_week(week_starts_on);
            let last_of_grid = month.end_of_month().end_of_week(week_starts_on);
            let mut days: Vec<CalendarDate> = dates
                .iter()
                .copied()
                .filter(|date| *date >= first_of_grid && *date <= last_of_grid)
                .collect();
            if fixed && days.len() < 42 {
                let limit = last_of_grid.add_days(i32::try_from(42 - days.len()).unwrap_or(0));
                days.extend(
                    dates
                        .iter()
                        .copied()
                        .filter(|date| *date > last_of_grid && *date <= limit),
                );
            }
            let mut weeks: Vec<Week> = Vec::new();
            for date in days {
                let number = state.week_number(date);
                let day = CalendarDay::new(date, month);
                match weeks.iter_mut().find(|week| week.number == number) {
                    Some(week) => week.days.push(day),
                    None => weeks.push(Week {
                        number,
                        days: vec![day],
                    }),
                }
            }
            Month { month, weeks }
        })
        .collect();
    if state.reverse_months.get() {
        months.reverse();
    }
    months
}

// the day the tab key lands on
fn focus_target(state: State) -> Option<CalendarDay> {
    let last_focused = state.last_focused.get();
    let today = state.today.get();
    let mut target = None;
    let mut priority = 0;
    let months = state.months.get();
    let days = || {
        months
            .iter()
            .flat_map(|month| &month.weeks)
            .flat_map(|week| &week.days)
    };
    for day in days() {
        if !state.focusable(*day) {
            continue;
        }
        if last_focused == Some(*day) && priority < 3 {
            target = Some(*day);
            priority = 3;
        } else if state.is_selected(day.date) && priority < 2 {
            target = Some(*day);
            priority = 2;
        } else if day.date == today && priority < 1 {
            target = Some(*day);
            priority = 1;
        }
    }
    target.or_else(|| days().copied().find(|day| state.focusable(*day)))
}

fn previous_month(this: State) -> Option<CalendarDate> {
    if this.disable_navigation.get() {
        return None;
    }
    let first = this.first_month().start_of_month();
    let offset = if this.paged_navigation.get() {
        i32::try_from(this.number_of_months.get()).unwrap_or(1)
    } else {
        1
    };
    match this.nav.get().0 {
        Some(start) if first.months_since(start) <= 0 => None,
        _ => Some(first.add_months(-offset)),
    }
}

fn next_month(this: State) -> Option<CalendarDate> {
    if this.disable_navigation.get() {
        return None;
    }
    let first = this.first_month().start_of_month();
    let count = i32::try_from(this.number_of_months.get())
        .unwrap_or(1)
        .max(1);
    let offset = if this.paged_navigation.get() {
        count
    } else {
        1
    };
    match this.nav.get().1 {
        Some(end) if end.months_since(first) < count => None,
        _ => Some(first.add_months(offset)),
    }
}

#[component(element = "div")]
pub fn DayPicker(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional_no_strip, into)] class_names: Option<Signal<ClassNames>>,
    #[prop(optional_no_strip, into)] mode: Option<Signal<DayPickerMode>>,
    #[prop(optional_no_strip, into)] selected: Option<Signal<Option<CalendarDate>>>,
    #[prop(optional_no_strip, into)] on_select: Option<Callback<Option<CalendarDate>>>,
    #[prop(optional_no_strip, into)] selected_dates: Option<Signal<Vec<CalendarDate>>>,
    #[prop(optional_no_strip, into)] on_select_dates: Option<Callback<Vec<CalendarDate>>>,
    #[prop(optional_no_strip, into)] selected_range: Option<Signal<Option<DateRange>>>,
    #[prop(optional_no_strip, into)] on_select_range: Option<Callback<Option<DateRange>>>,
    #[prop(optional_no_strip, into)] required: Option<Signal<bool>>,
    #[prop(optional_no_strip, into)] min: Option<Signal<u32>>,
    #[prop(optional_no_strip, into)] max: Option<Signal<u32>>,
    #[prop(optional, into)] exclude_disabled: Signal<bool>,
    #[prop(optional, into)] reset_on_select: Signal<bool>,
    #[prop(optional_no_strip, into)] month: Option<Signal<CalendarDate>>,
    #[prop(optional_no_strip, into)] default_month: Option<CalendarDate>,
    #[prop(optional_no_strip, into)] on_month_change: Option<Callback<CalendarDate>>,
    #[prop(optional_no_strip, into)] start_month: Option<Signal<CalendarDate>>,
    #[prop(optional_no_strip, into)] end_month: Option<Signal<CalendarDate>>,
    #[prop(into, default = 1.into())] number_of_months: Signal<u32>,
    #[prop(optional, into)] reverse_months: Signal<bool>,
    #[prop(optional, into)] paged_navigation: Signal<bool>,
    #[prop(optional, into)] disable_navigation: Signal<bool>,
    #[prop(optional, into)] hide_navigation: Signal<bool>,
    #[prop(optional_no_strip, into)] nav_layout: Option<Signal<NavLayout>>,
    #[prop(optional, into)] caption_layout: Signal<CaptionLayout>,
    #[prop(optional, into)] reverse_years: Signal<bool>,
    #[prop(optional, into)] show_outside_days: Signal<bool>,
    #[prop(optional, into)] show_week_number: Signal<bool>,
    #[prop(optional, into)] fixed_weeks: Signal<bool>,
    #[prop(optional, into)] hide_weekdays: Signal<bool>,
    #[prop(optional, into)] iso_week: Signal<bool>,
    #[prop(optional_no_strip, into)] week_starts_on: Option<Signal<Weekday>>,
    #[prop(optional_no_strip, into)] first_week_contains_date: Option<Signal<u8>>,
    #[prop(optional_no_strip, into)] disabled: Option<Signal<Matcher>>,
    #[prop(optional_no_strip, into)] hidden: Option<Signal<Matcher>>,
    #[prop(optional_no_strip, into)] modifiers: Option<Signal<Vec<(&'static str, Matcher)>>>,
    #[prop(optional_no_strip, into)] modifiers_class_names: Option<
        Signal<Vec<(&'static str, String)>>,
    >,
    #[prop(optional_no_strip, into)] today: Option<Signal<CalendarDate>>,
    #[prop(optional, into)] locale: Signal<Locale>,
    #[prop(optional, into)] labels: Signal<Labels>,
    #[prop(optional_no_strip, into)] dir: Option<Signal<Direction>>,
    #[prop(optional_no_strip, into)] lang: Option<Signal<String>>,
    #[prop(optional)] auto_focus: bool,
    #[prop(optional)] formatters: Formatters,
    #[prop(optional)] components: DayPickerComponents,
    #[prop(optional_no_strip, into)] footer: Option<ViewFn>,
) -> impl IntoView {
    let today = today.unwrap_or_else(|| {
        let today = RwSignal::new(CalendarDate::today_utc());
        // the server's html and the hydration agree on the utc date, then the browser's own takes over
        Effect::new(move |_| {
            let local = CalendarDate::today_local();
            if today.get_untracked() != local {
                today.set(local);
            }
        });
        today.into()
    });
    let bounds = Memo::new(move |_| {
        let today = today.get();
        let years = caption_layout.get().years_dropdown();
        let start = start_month
            .map(|start| start.get().start_of_month())
            .or_else(|| years.then(|| today.add_years(-100).start_of_year()));
        let end = end_month
            .map(|end| end.get().end_of_month())
            .or_else(|| years.then(|| today.end_of_year()));
        (start, end)
    });
    let state = StoredValue::new(State {
        mode,
        single: Choice::new(selected, on_select, None),
        multiple: Choice::new(selected_dates, on_select_dates, Vec::new()),
        range: Choice::new(selected_range, on_select_range, None),
        required,
        min,
        max,
        exclude_disabled,
        reset_on_select,
        month,
        owned_month: RwSignal::new(CalendarDate::min()),
        default_month,
        on_month_change,
        end_month,
        number_of_months,
        reverse_months,
        paged_navigation,
        disable_navigation,
        caption_layout,
        reverse_years,
        show_outside_days,
        fixed_weeks,
        iso_week,
        week_starts_on,
        first_week_contains_date,
        disabled,
        hidden,
        modifiers,
        modifiers_class_names,
        today,
        locale,
        labels,
        dir,
        class_names,
        formatters,
        components,
        focused: RwSignal::new(None),
        last_focused: RwSignal::new(None),
        nav: bounds,
        months: Memo::new(|_| Vec::new()),
        previous_month: Memo::new(|_| None),
        next_month: Memo::new(|_| None),
        focus_target: Memo::new(|_| None),
        root: NodeRef::new(),
    });
    let context = DayPickerContext(state);
    let months = Memo::new(move |_| {
        context
            .with(|this| layout(this, this.first_month()))
            .unwrap_or_default()
    });
    let previous = Memo::new(move |_| context.with(previous_month).flatten());
    let next = Memo::new(move |_| context.with(next_month).flatten());
    let target = Memo::new(move |_| context.with(focus_target).flatten());
    state.update_value(|state| {
        state.months = months;
        state.previous_month = previous;
        state.next_month = next;
        state.focus_target = target;
    });
    let this = state.get_value();
    this.owned_month.set(untrack(|| this.initial_month(None)));
    if auto_focus {
        this.focused.set(target.get_untracked());
        Effect::new(move |_| {
            if let Some(day) = target.get_untracked() {
                this.focus_day(day);
            }
        });
    }

    let root = this.root;
    let root_class = move || {
        let own = context.class(|names| &names.root, "rdp-root");
        match class {
            Some(class) => format!("{own} {}", class.get()),
            None => own,
        }
    };
    view! {
        <div
            {..attrs}
            node_ref=root
            class=root_class
            dir=move || dir.map(|dir| dir.get().as_str())
            lang=move || lang.map_or_else(|| locale.with(ToString::to_string), |lang| lang.get())
            data-mode=move || mode.map(|mode| mode.get().as_str())
            data-required=move || required.map(|required| required.get().to_string())
            data-multiple-months=move || (number_of_months.get() > 1).then_some("true")
            data-week-numbers=move || show_week_number.get().then_some("true")
            data-nav-layout=move || nav_layout.map(|layout| layout.get().as_str())
        >
            <Provider value=context>
                <div class=move || context.class(|names| &names.months, "rdp-months")>
                    {move || (!hide_navigation.get() && nav_layout.is_none()).then(|| nav(context))}
                    <For each=move || 0..months.with(Vec::len) key=|index| *index let:index>
                        {month_view(context, index, nav_layout, hide_navigation, show_week_number, hide_weekdays)}
                    </For>
                </div>
                {footer
                    .map(|footer| {
                        view! {
                            <div
                                class=move || context.class(|names| &names.footer, "rdp-footer")
                                role="status"
                                aria-live="polite"
                            >
                                {footer.run()}
                            </div>
                        }
                    })}
            </Provider>
        </div>
    }
}

fn chevron(
    context: DayPickerContext,
    orientation: ChevronOrientation,
    disabled: Signal<bool>,
    size: u32,
) -> AnyView {
    let class = context.class(|names| &names.chevron, "rdp-chevron");
    if let Some(chevron) = context.with(|this| this.components.chevron).flatten() {
        return chevron.run(ChevronProps {
            orientation,
            class,
            disabled,
            size,
        });
    }
    let points = match orientation {
        ChevronOrientation::Up => "6.77 17 12.5 11.43 18.24 17 20 15.28 12.5 8 5 15.28",
        ChevronOrientation::Down => "6.77 8 12.5 13.57 18.24 8 20 9.72 12.5 17 5 9.72",
        ChevronOrientation::Left => {
            "16 18.112 9.81111111 12 16 5.87733333 14.0888889 4 6 12 14.0888889 20"
        }
        ChevronOrientation::Right => {
            "8 18.112 14.18888889 12 8 5.87733333 9.91111111 4 18 12 9.91111111 20"
        }
    };
    view! {
        <svg class=class width=size height=size viewBox="0 0 24 24">
            <polygon points=points />
        </svg>
    }
    .into_any()
}

fn nav_button(
    context: DayPickerContext,
    next: bool,
    orientation: ChevronOrientation,
) -> impl IntoView {
    let target = context
        .with(|this| {
            if next {
                this.next_month
            } else {
                this.previous_month
            }
        })
        .unwrap_or_else(|| Memo::new(|_| None));
    let disabled = Signal::derive(move || target.get().is_none());
    let (pick, default): (Pick, &'static str) = if next {
        (|names| &names.button_next, "rdp-button_next")
    } else {
        (|names| &names.button_previous, "rdp-button_previous")
    };
    let label = move || {
        context.with(|this| {
            this.labels.with(|labels| {
                if next {
                    labels.next.clone()
                } else {
                    labels.previous.clone()
                }
            })
        })
    };
    let click = move |_: ev::MouseEvent| {
        if let Some(month) = target.get_untracked() {
            context.with(|this| this.go_to_month(month));
        }
    };
    view! {
        <button
            type="button"
            class=move || context.class(pick, default)
            tabindex=move || disabled.get().then_some(-1)
            aria-disabled=move || disabled.get().then_some("true")
            aria-label=label
            on:click=click
        >
            {move || chevron(context, orientation, disabled, 24)}
        </button>
    }
}

fn nav(context: DayPickerContext) -> impl IntoView {
    view! {
        <nav
            class=move || context.class(|names| &names.nav, "rdp-nav")
            aria-label=move || context.with(|this| this.labels.with(|labels| labels.nav.clone()))
        >
            {nav_button(context, false, ChevronOrientation::Left)}
            {nav_button(context, true, ChevronOrientation::Right)}
        </nav>
    }
}

fn shown_month(context: DayPickerContext, index: usize) -> Memo<CalendarDate> {
    Memo::new(move |_| {
        context
            .with(|this| {
                this.months
                    .with(|months| months.get(index).map(|month| month.month))
            })
            .flatten()
            .unwrap_or_else(CalendarDate::min)
    })
}

fn dropdown_options(this: State, month: CalendarDate, years: bool) -> Vec<(i32, String, bool)> {
    let (start, end) = this.nav.get();
    if years {
        let (Some(start), Some(end)) = (start, end) else {
            return Vec::new();
        };
        let mut options: Vec<_> = (start.year()..=end.year())
            .map(|year| {
                let label = this.format_year_dropdown(month.with_year(year).start_of_year());
                (year, label, false)
            })
            .collect();
        if this.reverse_years.get() {
            options.reverse();
        }
        return options;
    }
    (1..=12)
        .map(|number| {
            let option = month.start_of_year().with_month(number);
            let disabled = start.is_some_and(|start| option < start.start_of_month())
                || end.is_some_and(|end| option > end.start_of_month());
            (
                i32::from(number) - 1,
                this.format_month_dropdown(option),
                disabled,
            )
        })
        .collect()
}

fn dropdown(context: DayPickerContext, index: usize, years: bool) -> impl IntoView {
    let month = shown_month(context, index);
    let offset = move || {
        let index = i32::try_from(index).unwrap_or(0);
        let reversed = context.with(|this| {
            this.reverse_months
                .get_untracked()
                .then(|| this.months.with_untracked(Vec::len))
        });
        match reversed.flatten() {
            Some(count) => i32::try_from(count).unwrap_or(1) - 1 - index,
            None => index,
        }
    };
    let value = move || {
        let month = month.get();
        if years {
            month.year()
        } else {
            i32::from(month.month()) - 1
        }
    };
    let options = move || {
        context
            .with(|this| dropdown_options(this, month.get(), years))
            .unwrap_or_default()
    };
    let (pick, default): (Pick, &'static str) = if years {
        (|names| &names.years_dropdown, "rdp-years_dropdown")
    } else {
        (|names| &names.months_dropdown, "rdp-months_dropdown")
    };
    let label = move || {
        context.with(|this| {
            this.labels.with(|labels| {
                if years {
                    labels.year_dropdown.clone()
                } else {
                    labels.month_dropdown.clone()
                }
            })
        })
    };
    let shown = move || {
        let value = value();
        options()
            .into_iter()
            .find(|(option, _, _)| *option == value)
            .map(|(_, label, _)| label)
    };
    let disabled = move || {
        context
            .with(|this| this.disable_navigation.get())
            .unwrap_or_default()
    };
    let change = move |event: ev::Event| {
        let value = event_target_value(&event);
        context.with(|this| this.dropdown_change(month.get_untracked(), offset(), &value, years));
    };
    view! {
        <span
            data-disabled=move || disabled().to_string()
            class=move || context.class(|names| &names.dropdown_root, "rdp-dropdown_root")
        >
            <select
                class=move || {
                    format!(
                        "{} {}",
                        context.class(|names| &names.dropdown, "rdp-dropdown"),
                        context.class(pick, default),
                    )
                }
                aria-label=label
                disabled=disabled
                prop:value=move || value().to_string()
                on:change=change
            >
                {move || {
                    let value = value();
                    options()
                        .into_iter()
                        .map(|(option, label, disabled)| {
                            view! {
                                <option value=option selected=option == value disabled=disabled>
                                    {label}
                                </option>
                            }
                        })
                        .collect_view()
                }}
            </select>
            <span
                class=move || context.class(|names| &names.caption_label, "rdp-caption_label")
                aria-hidden="true"
            >
                {shown}
                {move || chevron(context, ChevronOrientation::Down, Signal::stored(false), 18)}
            </span>
        </span>
    }
}

fn caption(context: DayPickerContext, index: usize) -> impl IntoView {
    let month = shown_month(context, index);
    let text = move || {
        context
            .with(|this| this.format_caption(month.get()))
            .unwrap_or_default()
    };
    let layout = Memo::new(move |_| {
        context
            .with(|this| this.caption_layout.get())
            .unwrap_or_default()
    });
    let plain = move |years: bool| {
        context
            .with(|this| {
                let month = month.get();
                if years {
                    this.format_year_dropdown(month)
                } else {
                    this.format_month_dropdown(month)
                }
            })
            .unwrap_or_default()
    };
    move || {
        let layout = layout.get();
        if layout == CaptionLayout::Label {
            return view! {
                <span
                    class=move || context.class(|names| &names.caption_label, "rdp-caption_label")
                    role="status"
                    aria-live="polite"
                >
                    {text}
                </span>
            }
            .into_any();
        }
        let control = move |years: bool| {
            let wanted = if years {
                layout.years_dropdown()
            } else {
                layout.months_dropdown()
            };
            if wanted {
                dropdown(context, index, years).into_any()
            } else {
                view! { <span>{move || plain(years)}</span> }.into_any()
            }
        };
        view! {
            <div class=move || context.class(|names| &names.dropdowns, "rdp-dropdowns")>
                {control(false)}
                {control(true)}
                <span role="status" aria-live="polite" style=STATUS_STYLE>
                    {text}
                </span>
            </div>
        }
        .into_any()
    }
}

fn day_button(context: DayPickerContext, day: CalendarDay, modifiers: Memo<Modifiers>) -> AnyView {
    let Some(this) = context.state() else {
        return ().into_any();
    };
    let class = this.class(|names| &names.day_button, "rdp-day_button");
    let children = this.format_day(day.date);
    match this.components.day_button {
        Some(slot) => slot.run(DayButtonProps {
            day,
            modifiers: modifiers.into(),
            class,
            children,
        }),
        None => {
            view! { <DayPickerDayButton class=class>{children}</DayPickerDayButton> }.into_any()
        }
    }
}

fn day_view(context: DayPickerContext, day: CalendarDay) -> impl IntoView {
    let modifiers =
        Memo::new(move |_| context.with(|this| this.modifiers(day)).unwrap_or_default());
    let hidden = Memo::new(move |_| modifiers.with(|modifiers| modifiers.hidden));
    let interactive = context.with(|this| this.mode.is_some()).unwrap_or_default();
    let flag = move |get: fn(&Modifiers) -> bool| move || modifiers.with(get).then_some("true");
    let shown = DayPickerDay {
        day,
        modifiers: modifiers.into(),
        locale: context.with(|this| this.locale).unwrap_or_default(),
    };
    let inner = move || {
        if hidden.get() {
            return None;
        }
        if !interactive {
            let text = untrack(|| context.with(|this| this.format_day(day.date)));
            return Some(text.into_any());
        }
        Some(
            view! { <Provider value=shown>{untrack(|| day_button(context, day, modifiers))}</Provider> }
                .into_any(),
        )
    };
    let label = move || {
        (!interactive && !hidden.get())
            .then(|| {
                modifiers.with(|modifiers| {
                    context.with(|this| this.day_label(day.date, modifiers, false))
                })
            })
            .flatten()
    };
    view! {
        <td
            class=move || {
                modifiers
                    .with(|modifiers| context.with(|this| this.day_class(modifiers)))
                    .unwrap_or_default()
            }
            role="gridcell"
            aria-selected=flag(|modifiers| modifiers.selected)
            aria-label=label
            data-day=day.date.to_string()
            data-month=day
                .outside
                .then(|| format!("{:04}-{:02}", day.date.year(), day.date.month()))
            data-selected=flag(|modifiers| modifiers.selected)
            data-disabled=flag(|modifiers| modifiers.disabled)
            data-hidden=flag(|modifiers| modifiers.hidden)
            data-outside=day.outside.then_some("true")
            data-focused=flag(|modifiers| modifiers.focused)
            data-today=flag(|modifiers| modifiers.today)
        >
            {inner}
        </td>
    }
}

fn week_number_view(this: State, number: i32) -> AnyView {
    let class = this.class(|names| &names.week_number, "rdp-week_number");
    let label = this
        .labels
        .with_untracked(|labels| format!("{} {number}", labels.week));
    let children = this.format_week_number(number);
    if let Some(slot) = this.components.week_number {
        return slot.run(WeekNumberProps {
            week_number: number,
            class,
            label,
            children,
        });
    }
    view! {
        <th class=class aria-label=label scope="row" role="rowheader">
            {children}
        </th>
    }
    .into_any()
}

fn weekdays_view(context: DayPickerContext, show_week_number: Signal<bool>) -> impl IntoView {
    let header = move || {
        show_week_number.get().then(|| {
            let header = context
                .with(|this| {
                    this.formatters
                        .format_week_number_header
                        .map_or_else(String::new, |format| format.run(()))
                })
                .unwrap_or_default();
            view! {
                <th
                    aria-label=move || context.with(|this| this.labels.with(|labels| labels.week_number_header.clone()))
                    class=move || context.class(|names| &names.week_number_header, "rdp-week_number_header")
                    scope="col"
                >
                    {header}
                </th>
            }
        })
    };
    let days = move || {
        let Some(this) = context.state() else {
            return Vec::new();
        };
        let first = this.week_starts_on();
        let locale = this.locale.get();
        (0..7)
            .map(|offset| {
                let weekday = Weekday::from_index(first.index() + offset);
                view! {
                    <th
                        aria-label=locale.weekday(weekday)
                        class=move || context.class(|names| &names.weekday, "rdp-weekday")
                        scope="col"
                    >
                        {this.format_weekday(weekday)}
                    </th>
                }
            })
            .collect::<Vec<_>>()
    };
    view! {
        <thead aria-hidden="true">
            <tr class=move || context.class(|names| &names.weekdays, "rdp-weekdays")>
                {header}
                {days}
            </tr>
        </thead>
    }
}

fn month_view(
    context: DayPickerContext,
    index: usize,
    nav_layout: Option<Signal<NavLayout>>,
    hide_navigation: Signal<bool>,
    show_week_number: Signal<bool>,
    hide_weekdays: Signal<bool>,
) -> impl IntoView {
    let grid = Memo::new(move |_| {
        context
            .with(|this| this.months.with(|months| months.get(index).cloned()))
            .flatten()
    });
    let last = move || {
        context
            .with(|this| index + 1 == usize::try_from(this.number_of_months.get()).unwrap_or(1))
            .unwrap_or_default()
    };
    let shown_layout = move || nav_layout.map(|layout| layout.get());
    let around = move |first: bool| {
        let wanted = !hide_navigation.get()
            && shown_layout() == Some(NavLayout::Around)
            && if first { index == 0 } else { last() };
        wanted.then(|| {
            let rtl = context
                .with(|this| {
                    this.dir
                        .is_some_and(|dir| dir.get_untracked() == Direction::Rtl)
                })
                .unwrap_or_default();
            let orientation = if first == rtl {
                ChevronOrientation::Right
            } else {
                ChevronOrientation::Left
            };
            nav_button(context, !first, orientation)
        })
    };
    let multiselectable = move || {
        context
            .with(|this| {
                matches!(
                    this.mode(),
                    Some(DayPickerMode::Multiple | DayPickerMode::Range)
                )
            })
            .unwrap_or_default()
            .to_string()
    };
    let label = move || {
        let month = grid.with(|grid| grid.as_ref().map(|grid| grid.month))?;
        context.with(|this| this.locale.with(|locale| locale.month_year(month)))
    };
    let weeks = move || {
        let numbers = show_week_number.get();
        let Some(this) = context.state() else {
            return Vec::new();
        };
        grid.get()
            .map(|grid| grid.weeks)
            .unwrap_or_default()
            .into_iter()
            .map(|week| {
                let number = numbers.then(|| week_number_view(this, week.number));
                let days = week
                    .days
                    .into_iter()
                    .map(|day| day_view(context, day))
                    .collect_view();
                view! {
                    <tr class=move || context.class(|names| &names.week, "rdp-week")>
                        {number}
                        {days}
                    </tr>
                }
            })
            .collect::<Vec<_>>()
    };
    view! {
        <div class=move || context.class(|names| &names.month, "rdp-month")>
            {move || around(true)}
            <div class=move || context.class(|names| &names.month_caption, "rdp-month_caption")>
                {caption(context, index)}
            </div>
            {move || around(false)}
            {move || {
                (last() && shown_layout() == Some(NavLayout::After) && !hide_navigation.get())
                    .then(|| nav(context))
            }}
            <table
                role="grid"
                aria-multiselectable=multiselectable
                aria-label=label
                class=move || context.class(|names| &names.month_grid, "rdp-month_grid")
            >
                {move || (!hide_weekdays.get()).then(|| weekdays_view(context, show_week_number))}
                <tbody class=move || context.class(|names| &names.weeks, "rdp-weeks")>{weeks}</tbody>
            </table>
        </div>
    }
}

#[component(element = "button", skip(disabled, tabindex, r#type))]
pub fn DayPickerDayButton(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<DayPickerContext>();
    let DayPickerDay { day, modifiers, .. } = use_day_picker_day();
    let click = move |event: ev::MouseEvent| {
        event.prevent_default();
        event.stop_propagation();
        context.with(|this| {
            this.focused.set(Some(day));
            if !modifiers.with_untracked(|modifiers| modifiers.disabled) {
                this.select(day.date);
            }
        });
    };
    let blur = move |_: ev::FocusEvent| {
        context.with(|this| {
            this.last_focused.set(this.focused.get_untracked());
            this.focused.set(None);
        });
    };
    let tabindex = move || {
        let target = context.with(|this| this.focus_target.get()).flatten();
        if target == Some(day) { 0 } else { -1 }
    };
    let label = move || {
        modifiers
            .with(|modifiers| context.with(|this| this.day_label(day.date, modifiers, true)))
            .unwrap_or_default()
    };
    view! {
        <button
            {..attrs}
            type="button"
            class=move || class.map(|class| class.get())
            disabled=move || modifiers.with(|modifiers| !modifiers.focused && modifiers.disabled)
            aria-disabled=move || {
                modifiers.with(|modifiers| (modifiers.focused && modifiers.disabled).then_some("true"))
            }
            tabindex=tabindex
            aria-label=label
            on:click=click
            on:focus=move |_| {
                context.with(|this| this.focused.set(Some(day)));
            }
            on:blur=blur
            on:keydown=move |event| {
                context.with(|this| this.key_down(&event));
            }
        >
            {children.map(|children| children())}
        </button>
    }
}
