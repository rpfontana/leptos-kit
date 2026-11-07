use icu_calendar::options::{DateAddOptions, Overflow};
use icu_calendar::types::{DateDuration, RataDie};
use icu_calendar::{Date, Gregorian, Iso};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

// icu4x's gregorian date, kept to the years -9999 to 9999 so chrono, time and jiff all hold it
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CalendarDate(Date<Gregorian>);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Weekday {
    #[default]
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl Weekday {
    const ALL: [Self; 7] = [
        Self::Sunday,
        Self::Monday,
        Self::Tuesday,
        Self::Wednesday,
        Self::Thursday,
        Self::Friday,
        Self::Saturday,
    ];

    // sunday is 0, as js's getDay
    pub(crate) const fn index(self) -> i32 {
        match self {
            Self::Sunday => 0,
            Self::Monday => 1,
            Self::Tuesday => 2,
            Self::Wednesday => 3,
            Self::Thursday => 4,
            Self::Friday => 5,
            Self::Saturday => 6,
        }
    }

    pub(crate) fn from_index(index: i32) -> Self {
        Self::ALL[usize::try_from(index.rem_euclid(7)).unwrap_or_default()]
    }

    pub(crate) fn from_icu(weekday: icu_calendar::types::Weekday) -> Self {
        use icu_calendar::types::Weekday as Icu;
        match weekday {
            Icu::Sunday => Self::Sunday,
            Icu::Monday => Self::Monday,
            Icu::Tuesday => Self::Tuesday,
            Icu::Wednesday => Self::Wednesday,
            Icu::Thursday => Self::Thursday,
            Icu::Friday => Self::Friday,
            Icu::Saturday => Self::Saturday,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DateRange {
    pub from: Option<CalendarDate>,
    pub to: Option<CalendarDate>,
}

impl DateRange {
    pub const fn new(from: CalendarDate, to: CalendarDate) -> Self {
        Self {
            from: Some(from),
            to: Some(to),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseCalendarDateError;

impl fmt::Display for ParseCalendarDateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid calendar date, expected yyyy-mm-dd")
    }
}

impl std::error::Error for ParseCalendarDateError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutOfRangeError;

impl fmt::Display for OutOfRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("date out of the years -9999 to 9999")
    }
}

impl std::error::Error for OutOfRangeError {}

const YEARS: std::ops::RangeInclusive<i32> = -9999..=9999;

impl CalendarDate {
    pub fn try_new(year: i32, month: u8, day: u8) -> Option<Self> {
        if !YEARS.contains(&year) {
            return None;
        }
        Date::try_new_gregorian(year, month, day).ok().map(Self)
    }

    // panics on a date that doesn't exist, try_new doesn't
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        Self::try_new(year, month, day)
            .unwrap_or_else(|| panic!("{year}-{month}-{day} is not a calendar date"))
    }

    pub fn year(self) -> i32 {
        self.0.year().extended_year()
    }

    pub fn month(self) -> u8 {
        self.0.month().ordinal
    }

    pub fn day(self) -> u8 {
        self.0.day_of_month().0
    }

    pub fn weekday(self) -> Weekday {
        Weekday::from_icu(self.0.weekday())
    }

    #[must_use]
    pub fn add_days(self, days: i32) -> Self {
        let day = self.0.to_rata_die().to_i64_date() + i64::from(days);
        Self::from_rata_die(day)
    }

    // a day past the end of the month lands on its last, as date-fns's addMonths
    #[must_use]
    pub fn add_months(self, months: i32) -> Self {
        let mut options = DateAddOptions::default();
        options.overflow = Some(Overflow::Constrain);
        self.0
            .try_added_with_options(DateDuration::for_months(months), options)
            .ok()
            .filter(|date| YEARS.contains(&date.year().extended_year()))
            .map_or_else(|| if months < 0 { Self::min() } else { Self::max() }, Self)
    }

    #[must_use]
    pub fn add_years(self, years: i32) -> Self {
        self.add_months(years.saturating_mul(12))
    }

    // the date at greenwich: the server can't know the browser's time zone, so both start from it
    pub fn today_utc() -> Self {
        let millis = if cfg!(target_arch = "wasm32") {
            web_sys::js_sys::Date::now()
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0.0, |since| since.as_secs_f64() * 1000.0)
        };
        let epoch = Self::new(1970, 1, 1).0.to_rata_die().to_i64_date();
        Self::from_rata_die(epoch + days_of_millis(millis))
    }

    // the browser's own date, the utc one off the browser
    pub fn today_local() -> Self {
        if !cfg!(target_arch = "wasm32") {
            return Self::today_utc();
        }
        let now = web_sys::js_sys::Date::new_0();
        let year = i32::try_from(now.get_full_year()).unwrap_or(1970);
        let month = u8::try_from(now.get_month() + 1).unwrap_or(1);
        let day = u8::try_from(now.get_date()).unwrap_or(1);
        Self::try_new(year, month, day).unwrap_or_else(Self::today_utc)
    }

    pub(crate) fn min() -> Self {
        Self::new(*YEARS.start(), 1, 1)
    }

    pub(crate) fn max() -> Self {
        Self::new(*YEARS.end(), 12, 31)
    }

    pub(crate) fn icu(self) -> Date<Gregorian> {
        self.0
    }

    fn from_rata_die(day: i64) -> Self {
        let first = Self::min().0.to_rata_die().to_i64_date();
        let last = Self::max().0.to_rata_die().to_i64_date();
        Self(Date::from_rata_die(
            RataDie::new(day.clamp(first, last)),
            Gregorian,
        ))
    }

    pub(crate) fn days_since(self, other: Self) -> i32 {
        let days = self.0.to_rata_die().to_i64_date() - other.0.to_rata_die().to_i64_date();
        i32::try_from(days).unwrap_or(if days < 0 { i32::MIN } else { i32::MAX })
    }

    pub(crate) fn months_since(self, other: Self) -> i32 {
        (self.year() - other.year()) * 12 + i32::from(self.month()) - i32::from(other.month())
    }

    #[must_use]
    pub fn start_of_month(self) -> Self {
        Self::try_new(self.year(), self.month(), 1).unwrap_or(self)
    }

    pub fn days_in_month(self) -> u8 {
        self.0.days_in_month()
    }

    pub(crate) fn end_of_month(self) -> Self {
        Self::try_new(self.year(), self.month(), self.days_in_month()).unwrap_or(self)
    }

    pub(crate) fn start_of_year(self) -> Self {
        Self::try_new(self.year(), 1, 1).unwrap_or(self)
    }

    pub(crate) fn end_of_year(self) -> Self {
        Self::try_new(self.year(), 12, 31).unwrap_or(self)
    }

    pub(crate) fn with_month(self, month: u8) -> Self {
        Self::try_new(self.year(), month, 1).unwrap_or(self)
    }

    pub(crate) fn with_year(self, year: i32) -> Self {
        Self::try_new(year, self.month(), 1).unwrap_or(self)
    }

    #[must_use]
    pub fn start_of_week(self, week_starts_on: Weekday) -> Self {
        self.add_days(-(self.weekday().index() - week_starts_on.index()).rem_euclid(7))
    }

    pub(crate) fn end_of_week(self, week_starts_on: Weekday) -> Self {
        self.start_of_week(week_starts_on).add_days(6)
    }

    // react-day-picker's week numbers: week one holds january's `first_week_contains_date`
    pub(crate) fn week(self, week_starts_on: Weekday, first_week_contains_date: u8) -> i32 {
        let start_of = |year: i32| {
            Self::try_new(year, 1, first_week_contains_date.clamp(1, 7))
                .unwrap_or_else(Self::min)
                .start_of_week(week_starts_on)
        };
        let year = self.year();
        let week_year = if self >= start_of(year + 1) {
            year + 1
        } else if self >= start_of(year) {
            year
        } else {
            year - 1
        };
        self.start_of_week(week_starts_on)
            .days_since(start_of(week_year))
            .div_euclid(7)
            + 1
    }

    pub(crate) fn iso_week(self) -> i32 {
        i32::from(self.0.to_calendar(Iso).week_of_year().week_number)
    }
}

#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "milliseconds since 1970 fit an i64 once divided into days"
)]
fn days_of_millis(millis: f64) -> i64 {
    (millis / 86_400_000.0).floor() as i64
}

impl Hash for CalendarDate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_rata_die().to_i64_date().hash(state);
    }
}

impl fmt::Display for CalendarDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let year = self.year();
        let sign = if year < 0 { "-" } else { "" };
        write!(
            f,
            "{sign}{:04}-{:02}-{:02}",
            year.abs(),
            self.month(),
            self.day()
        )
    }
}

impl FromStr for CalendarDate {
    type Err = ParseCalendarDateError;

    // the value of an `<input type="date">`
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (negative, text) = text
            .strip_prefix('-')
            .map_or((false, text), |rest| (true, rest));
        let mut parts = text.splitn(3, '-');
        let mut next = || parts.next().ok_or(ParseCalendarDateError);
        let (year, month, day) = (next()?, next()?, next()?);
        if year.len() != 4 || month.len() != 2 || day.len() != 2 {
            return Err(ParseCalendarDateError);
        }
        let year: i32 = year.parse().map_err(|_| ParseCalendarDateError)?;
        let month = month.parse().map_err(|_| ParseCalendarDateError)?;
        let day = day.parse().map_err(|_| ParseCalendarDateError)?;
        Self::try_new(if negative { -year } else { year }, month, day).ok_or(ParseCalendarDateError)
    }
}

#[cfg(feature = "chrono")]
mod chrono_conversions {
    use super::{CalendarDate, OutOfRangeError};
    use chrono::Datelike;

    impl From<CalendarDate> for chrono::NaiveDate {
        fn from(date: CalendarDate) -> Self {
            Self::from_ymd_opt(date.year(), date.month().into(), date.day().into())
                .expect("chrono's range holds every calendar date")
        }
    }

    impl TryFrom<chrono::NaiveDate> for CalendarDate {
        type Error = OutOfRangeError;

        fn try_from(date: chrono::NaiveDate) -> Result<Self, Self::Error> {
            let month = u8::try_from(date.month()).map_err(|_| OutOfRangeError)?;
            let day = u8::try_from(date.day()).map_err(|_| OutOfRangeError)?;
            Self::try_new(date.year(), month, day).ok_or(OutOfRangeError)
        }
    }
}

#[cfg(feature = "time")]
mod time_conversions {
    use super::{CalendarDate, OutOfRangeError};

    impl From<CalendarDate> for time::Date {
        fn from(date: CalendarDate) -> Self {
            let month =
                time::Month::try_from(date.month()).expect("a calendar date's month is 1 to 12");
            Self::from_calendar_date(date.year(), month, date.day())
                .expect("time's range holds every calendar date")
        }
    }

    impl TryFrom<time::Date> for CalendarDate {
        type Error = OutOfRangeError;

        fn try_from(date: time::Date) -> Result<Self, Self::Error> {
            Self::try_new(date.year(), u8::from(date.month()), date.day()).ok_or(OutOfRangeError)
        }
    }
}

#[cfg(feature = "jiff")]
mod jiff_conversions {
    use super::CalendarDate;

    impl From<CalendarDate> for jiff::civil::Date {
        fn from(date: CalendarDate) -> Self {
            let year = i16::try_from(date.year()).expect("a calendar date's year is -9999 to 9999");
            let month = i8::try_from(date.month()).expect("a calendar date's month is 1 to 12");
            let day = i8::try_from(date.day()).expect("a calendar date's day is 1 to 31");
            Self::new(year, month, day).expect("jiff's range is the calendar date's")
        }
    }

    impl From<jiff::civil::Date> for CalendarDate {
        fn from(date: jiff::civil::Date) -> Self {
            Self::new(
                date.year().into(),
                date.month().unsigned_abs(),
                date.day().unsigned_abs(),
            )
        }
    }
}
