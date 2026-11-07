use crate::date::{CalendarDate, Weekday};
use icu_calendar::week::WeekInformation;
use icu_calendar::{Date, Gregorian};
use icu_datetime::fieldsets::enums::DateFieldSet;
use icu_datetime::options::YearStyle;
use icu_datetime::pattern::{DateTimePattern, FixedCalendarDateTimeNames, WeekdayNameLength};
use icu_datetime::{DateTimeFormatterPreferences, FixedCalendarDateTimeFormatter, fieldsets};
use std::fmt;
use std::str::FromStr;
use std::sync::{Arc, LazyLock, OnceLock};
use writeable::TryWriteable;

type Formatter<F> = FixedCalendarDateTimeFormatter<Gregorian, F>;

// a bcp 47 locale with cldr's names and patterns from icu4x's data, each formatter made the first time it's needed
#[derive(Clone)]
pub struct Locale(Arc<Inner>);

struct Inner {
    tag: icu_locale_core::Locale,
    month_year: OnceLock<Option<Formatter<fieldsets::YM>>>,
    long_date: OnceLock<Option<Formatter<fieldsets::YMDE>>>,
    numeric_date: OnceLock<Option<Formatter<fieldsets::YMD>>>,
    month: OnceLock<Option<Formatter<fieldsets::M>>>,
    month_short: OnceLock<Option<Formatter<fieldsets::M>>>,
    year: OnceLock<Option<Formatter<fieldsets::Y>>>,
    weekday: OnceLock<Option<Formatter<fieldsets::E>>>,
    names: OnceLock<Option<FixedCalendarDateTimeNames<Gregorian, DateFieldSet>>>,
    first_weekday: OnceLock<Weekday>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseLocaleError;

impl fmt::Display for ParseLocaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid bcp 47 locale")
    }
}

impl std::error::Error for ParseLocaleError {}

static EN_US: LazyLock<Locale> =
    LazyLock::new(|| Locale::from_tag(icu_locale_core::locale!("en-US")));

// cldr has no field set for a day's number alone or for the shortest weekday ("Su")
static DAY: LazyLock<Option<DateTimePattern>> = LazyLock::new(|| "d".parse().ok());
static WEEKDAY_SHORT: LazyLock<Option<DateTimePattern>> = LazyLock::new(|| "EEEEEE".parse().ok());

fn sample(weekday: Weekday) -> CalendarDate {
    // 2026-11-01 was a sunday
    CalendarDate::new(2026, 11, 1).add_days(weekday.index())
}

// the iso date when icu4x has no data for it, so a calendar still reads
fn formatted<F>(
    formatter: Option<&F>,
    value: CalendarDate,
    format: impl FnOnce(&F, &Date<Gregorian>) -> String,
) -> String {
    formatter.map_or_else(
        || value.to_string(),
        |formatter| format(formatter, &value.icu()),
    )
}

impl Locale {
    fn from_tag(tag: icu_locale_core::Locale) -> Self {
        Self(Arc::new(Inner {
            tag,
            month_year: OnceLock::new(),
            long_date: OnceLock::new(),
            numeric_date: OnceLock::new(),
            month: OnceLock::new(),
            month_short: OnceLock::new(),
            year: OnceLock::new(),
            weekday: OnceLock::new(),
            names: OnceLock::new(),
            first_weekday: OnceLock::new(),
        }))
    }

    fn prefs(&self) -> DateTimeFormatterPreferences {
        (&self.0.tag).into()
    }

    fn pattern(&self, pattern: &LazyLock<Option<DateTimePattern>>, value: CalendarDate) -> String {
        let names = self.0.names.get_or_init(|| {
            let mut names = FixedCalendarDateTimeNames::try_new(self.prefs()).ok()?;
            names.include_weekday_names(WeekdayNameLength::Short).ok()?;
            names.include_decimal_formatter().ok()?;
            Some(names)
        });
        let (Some(names), Some(pattern)) = (names, pattern.as_ref()) else {
            return value.day().to_string();
        };
        names
            .with_pattern_unchecked(pattern)
            .format(&value.icu())
            .try_write_to_string()
            .map_or_else(|(_, text)| text.into_owned(), std::borrow::Cow::into_owned)
    }

    pub fn month_year(&self, value: CalendarDate) -> String {
        let formatter = self
            .0
            .month_year
            .get_or_init(|| Formatter::try_new(self.prefs(), fieldsets::YM::long()).ok());
        formatted(formatter.as_ref(), value, |formatter, date| {
            formatter.format(date).to_string()
        })
    }

    pub fn long_date(&self, value: CalendarDate) -> String {
        let formatter = self
            .0
            .long_date
            .get_or_init(|| Formatter::try_new(self.prefs(), fieldsets::YMDE::long()).ok());
        formatted(formatter.as_ref(), value, |formatter, date| {
            formatter.format(date).to_string()
        })
    }

    // as `toLocaleDateString`, "11/5/2026"
    pub fn numeric_date(&self, value: CalendarDate) -> String {
        let formatter = self.0.numeric_date.get_or_init(|| {
            let fields = fieldsets::YMD::short().with_year_style(YearStyle::Full);
            Formatter::try_new(self.prefs(), fields).ok()
        });
        formatted(formatter.as_ref(), value, |formatter, date| {
            formatter.format(date).to_string()
        })
    }

    pub fn month(&self, value: CalendarDate) -> String {
        let formatter = self
            .0
            .month
            .get_or_init(|| Formatter::try_new(self.prefs(), fieldsets::M::long()).ok());
        formatted(formatter.as_ref(), value, |formatter, date| {
            formatter.format(date).to_string()
        })
    }

    pub fn month_short(&self, value: CalendarDate) -> String {
        let formatter = self
            .0
            .month_short
            .get_or_init(|| Formatter::try_new(self.prefs(), fieldsets::M::medium()).ok());
        formatted(formatter.as_ref(), value, |formatter, date| {
            formatter.format(date).to_string()
        })
    }

    pub fn year(&self, value: CalendarDate) -> String {
        let formatter = self
            .0
            .year
            .get_or_init(|| Formatter::try_new(self.prefs(), fieldsets::Y::long()).ok());
        formatted(formatter.as_ref(), value, |formatter, date| {
            formatter.format(date).to_string()
        })
    }

    pub fn day(&self, value: CalendarDate) -> String {
        self.pattern(&DAY, value)
    }

    pub fn weekday(&self, weekday: Weekday) -> String {
        let formatter = self
            .0
            .weekday
            .get_or_init(|| Formatter::try_new(self.prefs(), fieldsets::E::long()).ok());
        formatted(formatter.as_ref(), sample(weekday), |formatter, date| {
            formatter.format(date).to_string()
        })
    }

    pub fn weekday_short(&self, weekday: Weekday) -> String {
        self.pattern(&WEEKDAY_SHORT, sample(weekday))
    }

    pub fn first_weekday(&self) -> Weekday {
        *self.0.first_weekday.get_or_init(|| {
            use icu_calendar::types::Weekday as Icu;
            match WeekInformation::try_new((&self.0.tag).into()).map(|week| week.first_weekday) {
                Ok(Icu::Sunday) => Weekday::Sunday,
                Ok(Icu::Tuesday) => Weekday::Tuesday,
                Ok(Icu::Wednesday) => Weekday::Wednesday,
                Ok(Icu::Thursday) => Weekday::Thursday,
                Ok(Icu::Friday) => Weekday::Friday,
                Ok(Icu::Saturday) => Weekday::Saturday,
                Ok(Icu::Monday) | Err(_) => Weekday::Monday,
            }
        })
    }
}

impl Default for Locale {
    fn default() -> Self {
        EN_US.clone()
    }
}

impl FromStr for Locale {
    type Err = ParseLocaleError;

    fn from_str(tag: &str) -> Result<Self, Self::Err> {
        tag.parse()
            .map(Self::from_tag)
            .map_err(|_| ParseLocaleError)
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0.tag, f)
    }
}

impl fmt::Debug for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Locale")
            .field(&self.0.tag.to_string())
            .finish()
    }
}

impl PartialEq for Locale {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0.tag == other.0.tag
    }
}

impl Eq for Locale {}

// react-day-picker's own strings, not in cldr: english unless the app translates them
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Labels {
    pub nav: String,
    pub previous: String,
    pub next: String,
    pub month_dropdown: String,
    pub year_dropdown: String,
    pub week_number_header: String,
    pub week: String,
    pub today: String,
    pub selected: String,
}

impl Default for Labels {
    fn default() -> Self {
        Self {
            nav: "Navigation bar".into(),
            previous: "Go to the Previous Month".into(),
            next: "Go to the Next Month".into(),
            month_dropdown: "Choose the Month".into(),
            year_dropdown: "Choose the Year".into(),
            week_number_header: "Week Number".into(),
            week: "Week".into(),
            today: "Today".into(),
            selected: "selected".into(),
        }
    }
}
