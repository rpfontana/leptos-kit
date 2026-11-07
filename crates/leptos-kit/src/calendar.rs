use super::button::{ButtonSize, ButtonVariant, button_classes};
use super::utils::classes;
use leptos::prelude::*;
use leptos_kit::icons::{Icon, semantic};
use leptos_kit::macros::component;
use leptos_kit::primitives::{self, ChevronOrientation, ChevronProps, WeekNumberProps};

pub use primitives::{
    CalendarDate, CalendarDay, CaptionLayout, ClassNames, DateRange, DayButtonProps,
    DayPickerComponents, DayPickerMode, Formatters, Labels, Locale, Matcher, Modifiers, NavLayout,
    Weekday,
};

fn own_class_names(
    variant: ButtonVariant,
    layout: CaptionLayout,
    week_numbers: bool,
) -> ClassNames {
    let own = |class: &str| Some(class.to_string());
    let nav_button = |hook: &str| {
        Some(format!(
            "{} size-(--cell-size) p-0 select-none aria-disabled:opacity-50 {hook}",
            button_classes(variant, ButtonSize::Default)
        ))
    };
    let first = if week_numbers {
        "[&:nth-child(2)[data-selected=true]_button]:rounded-l-(--cell-radius)"
    } else {
        "[&:first-child[data-selected=true]_button]:rounded-l-(--cell-radius)"
    };
    ClassNames {
        root: own("w-fit rdp-root"),
        months: own("relative flex flex-col gap-4 md:flex-row rdp-months"),
        month: own("flex w-full flex-col gap-4 rdp-month"),
        nav: own("absolute inset-x-0 top-0 flex w-full items-center justify-between gap-1 rdp-nav"),
        button_previous: nav_button("rdp-button_previous"),
        button_next: nav_button("rdp-button_next"),
        month_caption: own(
            "flex h-(--cell-size) w-full items-center justify-center px-(--cell-size) rdp-month_caption",
        ),
        dropdowns: own(
            "flex h-(--cell-size) w-full items-center justify-center gap-1.5 text-sm font-medium rdp-dropdowns",
        ),
        dropdown_root: own("cn-calendar-dropdown-root rdp-dropdown_root"),
        dropdown: own("absolute inset-0 bg-popover opacity-0 rdp-dropdown"),
        caption_label: Some(if layout == CaptionLayout::Label {
            "font-medium select-none cn-calendar-caption text-sm rdp-caption_label".to_string()
        } else {
            "font-medium select-none cn-calendar-caption-label rdp-caption_label".to_string()
        }),
        month_grid: own("w-full border-collapse rdp-month_grid"),
        weekdays: own("flex rdp-weekdays"),
        weekday: own(
            "flex-1 rounded-(--cell-radius) text-[0.8rem] font-normal text-muted-foreground select-none rdp-weekday",
        ),
        week: own("mt-2 flex w-full rdp-week"),
        week_number_header: own("w-(--cell-size) select-none rdp-week_number_header"),
        week_number: own("text-[0.8rem] text-muted-foreground select-none rdp-week_number"),
        day: Some(format!(
            "group/day relative aspect-square h-full w-full rounded-(--cell-radius) p-0 text-center select-none [&:last-child[data-selected=true]_button]:rounded-r-(--cell-radius) {first} rdp-day"
        )),
        range_start: own(
            "relative isolate z-0 rounded-l-(--cell-radius) bg-muted after:absolute after:inset-y-0 after:right-0 after:w-4 after:bg-muted rdp-range_start",
        ),
        range_middle: own("rounded-none rdp-range_middle"),
        range_end: own(
            "relative isolate z-0 rounded-r-(--cell-radius) bg-muted after:absolute after:inset-y-0 after:left-0 after:w-4 after:bg-muted rdp-range_end",
        ),
        today: own(
            "rounded-(--cell-radius) bg-muted text-foreground data-[selected=true]:rounded-none rdp-today",
        ),
        outside: own("text-muted-foreground aria-selected:text-muted-foreground rdp-outside"),
        disabled: own("text-muted-foreground opacity-50 rdp-disabled"),
        hidden: own("invisible rdp-hidden"),
        ..ClassNames::default()
    }
}

fn chevron(
    ChevronProps {
        orientation,
        class,
        disabled,
        ..
    }: ChevronProps,
) -> AnyView {
    let (icon, class) = match orientation {
        ChevronOrientation::Left => (
            semantic::CALENDAR_CHEVRON_LEFT,
            format!("cn-rtl-flip size-4 {class}"),
        ),
        ChevronOrientation::Right => (
            semantic::CALENDAR_CHEVRON_RIGHT,
            format!("cn-rtl-flip size-4 {class}"),
        ),
        // no size-4: upstream the caption's `[&>svg]:size-3.5` beats it, here it sits a layer below
        ChevronOrientation::Up | ChevronOrientation::Down => {
            (semantic::CALENDAR_CHEVRON_DOWN, class)
        }
    };
    // lucide puts every prop on its svg, `disabled` too
    view! { <Icon icon=icon class=class attr:disabled=move || disabled.get().then_some("") /> }
        .into_any()
}

fn day_button(props: DayButtonProps) -> AnyView {
    view! { <CalendarDayButton class=props.class>{props.children}</CalendarDayButton> }.into_any()
}

fn week_number(props: WeekNumberProps) -> AnyView {
    view! {
        <td
            {..leptos::attr::custom::custom_attribute("scope", "row")}
            class=props.class
            aria-label=props.label
            role="rowheader"
        >
            <div class="flex size-(--cell-size) items-center justify-center text-center">
                {props.children}
            </div>
        </td>
    }
    .into_any()
}

#[component(element = "div")]
pub fn Calendar(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] class_names: Option<Signal<ClassNames>>,
    #[prop(into, default = true.into())] show_outside_days: Signal<bool>,
    #[prop(optional, into)] caption_layout: Signal<CaptionLayout>,
    #[prop(into, default = ButtonVariant::Ghost.into())] button_variant: Signal<ButtonVariant>,
    #[prop(optional, into)] locale: Signal<Locale>,
    #[prop(optional, into)] labels: Signal<Labels>,
    #[prop(optional)] formatters: Formatters,
    #[prop(optional)] components: DayPickerComponents,
    #[prop(optional, into)] mode: Option<Signal<DayPickerMode>>,
    #[prop(optional, into)] selected: Option<Signal<Option<CalendarDate>>>,
    #[prop(optional, into)] on_select: Option<Callback<Option<CalendarDate>>>,
    #[prop(optional, into)] selected_dates: Option<Signal<Vec<CalendarDate>>>,
    #[prop(optional, into)] on_select_dates: Option<Callback<Vec<CalendarDate>>>,
    #[prop(optional, into)] selected_range: Option<Signal<Option<DateRange>>>,
    #[prop(optional, into)] on_select_range: Option<Callback<Option<DateRange>>>,
    #[prop(optional, into)] required: Option<Signal<bool>>,
    #[prop(optional, into)] min: Option<Signal<u32>>,
    #[prop(optional, into)] max: Option<Signal<u32>>,
    #[prop(optional, into)] exclude_disabled: Signal<bool>,
    #[prop(optional, into)] reset_on_select: Signal<bool>,
    #[prop(optional, into)] month: Option<Signal<CalendarDate>>,
    #[prop(optional, into)] default_month: Option<CalendarDate>,
    #[prop(optional, into)] on_month_change: Option<Callback<CalendarDate>>,
    #[prop(optional, into)] start_month: Option<Signal<CalendarDate>>,
    #[prop(optional, into)] end_month: Option<Signal<CalendarDate>>,
    #[prop(into, default = 1.into())] number_of_months: Signal<u32>,
    #[prop(optional, into)] reverse_months: Signal<bool>,
    #[prop(optional, into)] paged_navigation: Signal<bool>,
    #[prop(optional, into)] disable_navigation: Signal<bool>,
    #[prop(optional, into)] hide_navigation: Signal<bool>,
    #[prop(optional, into)] nav_layout: Option<Signal<NavLayout>>,
    #[prop(optional, into)] reverse_years: Signal<bool>,
    #[prop(optional, into)] show_week_number: Signal<bool>,
    #[prop(optional, into)] fixed_weeks: Signal<bool>,
    #[prop(optional, into)] hide_weekdays: Signal<bool>,
    #[prop(optional, into)] iso_week: Signal<bool>,
    #[prop(optional, into)] week_starts_on: Option<Signal<Weekday>>,
    #[prop(optional, into)] first_week_contains_date: Option<Signal<u8>>,
    #[prop(optional, into)] disabled: Option<Signal<Matcher>>,
    #[prop(optional, into)] hidden: Option<Signal<Matcher>>,
    #[prop(optional, into)] modifiers: Option<Signal<Vec<(&'static str, Matcher)>>>,
    #[prop(optional, into)] modifiers_class_names: Option<Signal<Vec<(&'static str, String)>>>,
    #[prop(optional, into)] today: Option<Signal<CalendarDate>>,
    #[prop(optional, into)] dir: Option<Signal<primitives::Direction>>,
    #[prop(optional, into)] lang: Option<Signal<String>>,
    #[prop(optional)] auto_focus: bool,
    #[prop(optional, into)] footer: Option<ViewFn>,
) -> impl IntoView {
    let names = Memo::new(move |_| {
        let own = own_class_names(
            button_variant.get(),
            caption_layout.get(),
            show_week_number.get(),
        );
        match class_names {
            Some(names) => names.get().or(own),
            None => own,
        }
    });
    let month_dropdown =
        Callback::new(move |date: CalendarDate| locale.with(|locale| locale.month_short(date)));
    let formatters = Formatters {
        format_month_dropdown: formatters.format_month_dropdown.or(Some(month_dropdown)),
        ..formatters
    };
    let components = DayPickerComponents {
        chevron: components.chevron.or(Some(Callback::new(chevron))),
        day_button: components.day_button.or(Some(Callback::new(day_button))),
        week_number: components.week_number.or(Some(Callback::new(week_number))),
    };
    view! {
        <primitives::DayPicker
            {..attrs}
            class=classes(
                r"cn-calendar group/calendar rtl:**:[.rdp-button\_next>svg]:rotate-180 rtl:**:[.rdp-button\_previous>svg]:rotate-180",
                class,
            )
            class_names=Signal::from(names)
            show_outside_days=show_outside_days
            caption_layout=caption_layout
            locale=locale
            labels=labels
            formatters=formatters
            components=components
            mode=mode
            selected=selected
            on_select=on_select
            selected_dates=selected_dates
            on_select_dates=on_select_dates
            selected_range=selected_range
            on_select_range=on_select_range
            required=required
            min=min
            max=max
            exclude_disabled=exclude_disabled
            reset_on_select=reset_on_select
            month=month
            default_month=default_month
            on_month_change=on_month_change
            start_month=start_month
            end_month=end_month
            number_of_months=number_of_months
            reverse_months=reverse_months
            paged_navigation=paged_navigation
            disable_navigation=disable_navigation
            hide_navigation=hide_navigation
            nav_layout=nav_layout
            reverse_years=reverse_years
            show_week_number=show_week_number
            fixed_weeks=fixed_weeks
            hide_weekdays=hide_weekdays
            iso_week=iso_week
            week_starts_on=week_starts_on
            first_week_contains_date=first_week_contains_date
            disabled=disabled
            hidden=hidden
            modifiers=modifiers
            modifiers_class_names=modifiers_class_names
            today=today
            dir=dir
            lang=lang
            auto_focus=auto_focus
            footer=footer
            attr:data-slot="calendar"
        />
    }
}

#[component(element = "button", skip(disabled, tabindex, r#type))]
pub fn CalendarDayButton(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let primitives::DayPickerDay {
        day,
        modifiers,
        locale,
    } = primitives::use_day_picker_day();
    let own = move || {
        format!(
            "{} cn-calendar-day-button rdp-day",
            button_classes(ButtonVariant::Ghost, ButtonSize::Icon)
        )
    };
    let selected_single = move || {
        modifiers.with(|modifiers| {
            modifiers.selected.then(|| {
                let in_range = [
                    modifiers.range_start,
                    modifiers.range_end,
                    modifiers.range_middle,
                ]
                .contains(&Some(true));
                (!in_range).to_string()
            })
        })
    };
    let range = move |flag: fn(&Modifiers) -> Option<bool>| {
        move || modifiers.with(|modifiers| flag(modifiers).map(|flag| flag.to_string()))
    };
    view! {
        <primitives::DayPickerDayButton
            {..attrs}
            class=classes(own, class)
            attr:data-slot="button"
            attr:data-day=move || locale.with(|locale| locale.numeric_date(day.date))
            attr:data-selected-single=selected_single
            attr:data-range-start=range(|modifiers| modifiers.range_start)
            attr:data-range-end=range(|modifiers| modifiers.range_end)
            attr:data-range-middle=range(|modifiers| modifiers.range_middle)
            attr:data-disabled=move || {
                modifiers.with(|modifiers| (!modifiers.focused && modifiers.disabled).then_some(""))
            }
        >
            {children.map(|children| children())}
        </primitives::DayPickerDayButton>
    }
}
