use crate::id::ElementId;
use crate::native::{document_order, observe_resize};
use crate::{ChangeReason, Controlled, use_id};
use leptos::attr::custom::custom_attribute;
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos_kit_macros::component;
use std::collections::HashMap;
use web_sys::{HtmlElement, ScrollIntoViewOptions, ScrollLogicalPosition};

const SCORE_CONTINUE_MATCH: f64 = 1.0;
const SCORE_SPACE_WORD_JUMP: f64 = 0.9;
const SCORE_NON_SPACE_WORD_JUMP: f64 = 0.8;
const SCORE_CHARACTER_JUMP: f64 = 0.17;
const SCORE_TRANSPOSITION: f64 = 0.1;
const PENALTY_SKIPPED: f64 = 0.999;
const PENALTY_CASE_MISMATCH: f64 = 0.9999;
const PENALTY_NOT_COMPLETE: f64 = 0.99;

const SR_ONLY: &str = "position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;\
     clip:rect(0, 0, 0, 0);white-space:nowrap;border-width:0";

pub type CommandFilter = Callback<(String, String, Vec<String>), f64>;

// cmdk's command-score, counted in utf-16 units as javascript does
pub fn command_score(value: &str, search: &str, keywords: &[String]) -> f64 {
    let string = if keywords.is_empty() {
        value.to_string()
    } else {
        format!("{value} {}", keywords.join(" "))
    };
    let string: Vec<u16> = string.encode_utf16().collect();
    let abbreviation: Vec<u16> = search.encode_utf16().collect();
    let score = Score {
        lower_string: formatted(&string),
        lower_abbreviation: formatted(&abbreviation),
        string,
        abbreviation,
    };
    score.inner(0, 0, &mut HashMap::new())
}

struct Score {
    string: Vec<u16>,
    abbreviation: Vec<u16>,
    lower_string: Vec<u16>,
    lower_abbreviation: Vec<u16>,
}

impl Score {
    fn inner(&self, from: usize, at: usize, memo: &mut HashMap<(usize, usize), f64>) -> f64 {
        if at >= self.abbreviation.len() {
            return if from == self.string.len() {
                SCORE_CONTINUE_MATCH
            } else {
                PENALTY_NOT_COMPLETE
            };
        }
        if let Some(&score) = memo.get(&(from, at)) {
            return score;
        }
        let wanted = self.lower_abbreviation[at];
        let following = self.lower_abbreviation.get(at + 1).copied();
        let mut high = 0.0;
        let mut found = find(&self.lower_string, wanted, from);
        while let Some(index) = found {
            let mut score = self.inner(index + 1, at + 1, memo);
            if score > high {
                if index == from {
                    score *= SCORE_CONTINUE_MATCH;
                } else if is_gap(unit(&self.string, index - 1)) {
                    score *= SCORE_NON_SPACE_WORD_JUMP;
                    let breaks = count(&self.string[from..index - 1], is_gap);
                    if breaks > 0 && from > 0 {
                        score *= PENALTY_SKIPPED.powi(breaks);
                    }
                } else if is_space(unit(&self.string, index - 1)) {
                    score *= SCORE_SPACE_WORD_JUMP;
                    let breaks = count(&self.string[from..index - 1], is_space);
                    if breaks > 0 && from > 0 {
                        score *= PENALTY_SKIPPED.powi(breaks);
                    }
                } else {
                    score *= SCORE_CHARACTER_JUMP;
                    if from > 0 {
                        score *= PENALTY_SKIPPED.powi(power(index - from));
                    }
                }
                if self.string[index] != self.abbreviation[at] {
                    score *= PENALTY_CASE_MISMATCH;
                }
            }
            let previous = index.checked_sub(1).map(|before| self.lower_string[before]);
            if (score < SCORE_TRANSPOSITION && previous == following)
                || (following == Some(wanted) && previous != Some(wanted))
            {
                let transposed = self.inner(index + 1, at + 2, memo) * SCORE_TRANSPOSITION;
                if transposed > score {
                    score = transposed;
                }
            }
            if score > high {
                high = score;
            }
            found = find(&self.lower_string, wanted, index + 1);
        }
        memo.insert((from, at), high);
        high
    }
}

fn formatted(units: &[u16]) -> Vec<u16> {
    units
        .iter()
        .map(|&unit| match char::from_u32(u32::from(unit)) {
            Some(c) if is_space(Some(c)) => u16::from(b' '),
            Some(c) => c
                .to_lowercase()
                .next()
                .and_then(|lower| u16::try_from(u32::from(lower)).ok())
                .unwrap_or(unit),
            None => unit,
        })
        .collect()
}

fn unit(units: &[u16], index: usize) -> Option<char> {
    units
        .get(index)
        .and_then(|&unit| char::from_u32(u32::from(unit)))
}

fn find(units: &[u16], wanted: u16, from: usize) -> Option<usize> {
    units
        .iter()
        .skip(from)
        .position(|&unit| unit == wanted)
        .map(|at| at + from)
}

fn count(units: &[u16], test: fn(Option<char>) -> bool) -> i32 {
    power(
        units
            .iter()
            .filter(|&&unit| test(char::from_u32(u32::from(unit))))
            .count(),
    )
}

fn power(n: usize) -> i32 {
    i32::try_from(n).unwrap_or(i32::MAX)
}

fn is_gap(c: Option<char>) -> bool {
    matches!(
        c,
        Some('\\' | '/' | '_' | '+' | '.' | '#' | '"' | '@' | '[' | '(' | '{' | '&')
    )
}

fn is_space(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_whitespace() || c == '-')
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Top {
    Item(usize),
    Group(usize),
}

#[derive(Clone, Copy)]
struct Entry {
    id: usize,
    element: NodeRef<html::Div>,
    element_id: ElementId<html::Div>,
    value: RwSignal<String>,
    keywords: StoredValue<Vec<String>>,
    group: Option<usize>,
    // cmdk leaves force-mounted items out of its count and its groups
    counted: bool,
    disabled: Signal<bool>,
    on_select: Option<Callback<String>>,
}

#[derive(Clone, Copy)]
struct Group {
    id: usize,
    element: NodeRef<html::Div>,
    heading: NodeRef<html::Div>,
}

#[derive(Clone, Copy)]
struct GroupScope {
    id: usize,
    forced: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct CommandContext(StoredValue<Command>);

#[derive(Clone, Copy)]
struct Command {
    input: NodeRef<html::Input>,
    list: NodeRef<html::Div>,
    label_id: ElementId<html::Label>,
    input_id: ElementId<html::Input>,
    list_id: ElementId<html::Div>,
    search: RwSignal<Option<Controlled<String>>>,
    query: Signal<String>,
    filtering: Signal<bool>,
    value: Controlled<String>,
    looping: Signal<bool>,
    disable_pointer_selection: Signal<bool>,
    vim_bindings: Signal<bool>,
    entries: RwSignal<Vec<Entry>>,
    groups: RwSignal<Vec<Group>>,
    next: StoredValue<usize>,
    scores: Memo<HashMap<usize, f64>>,
    moved: Memo<HashMap<usize, usize>>,
    first: Memo<Option<usize>>,
    settling: StoredValue<bool>,
}

impl CommandContext {
    fn command(self) -> Command {
        self.0.get_value()
    }

    fn try_command(self) -> Option<Command> {
        self.0.try_get_value()
    }

    fn next_id(self) -> usize {
        self.command()
            .next
            .try_update_value(|next| {
                *next += 1;
                *next - 1
            })
            .unwrap_or_default()
    }

    fn current(self) -> String {
        self.try_command()
            .map(|command| {
                command
                    .value
                    .with_untracked(|value| value.trim().to_string())
            })
            .unwrap_or_default()
    }

    fn shown(self, entry: &Entry) -> bool {
        self.try_command().is_some_and(|command| {
            !entry.counted
                || !command.filtering.get()
                || command
                    .scores
                    .with(|scores| scores.get(&entry.id).is_some_and(|&score| score > 0.0))
        })
    }

    fn group_shown(self, group: usize) -> bool {
        self.try_command().is_some_and(|command| {
            !command.filtering.get()
                || command.entries.with(|entries| {
                    entries.iter().any(|entry| {
                        entry.counted
                            && entry.group == Some(group)
                            && command.scores.with(|scores| {
                                scores.get(&entry.id).is_some_and(|&score| score > 0.0)
                            })
                    })
                })
        })
    }

    fn count(self) -> usize {
        self.try_command()
            .map(|command| {
                command.entries.with(|entries| {
                    entries
                        .iter()
                        .filter(|entry| entry.counted && self.shown(entry))
                        .count()
                })
            })
            .unwrap_or_default()
    }

    fn order(self, id: usize) -> Option<String> {
        self.try_command()?
            .moved
            .with(|moved| moved.get(&id).map(ToString::to_string))
    }

    fn layout(self) -> (Vec<Entry>, Vec<Group>) {
        let Some(command) = self.try_command() else {
            return (Vec::new(), Vec::new());
        };
        untrack(|| {
            arrange(
                command.entries.get(),
                command.groups.get(),
                command.filtering.get(),
                &command.scores.get(),
                &command.moved.get(),
            )
        })
    }

    fn valid(self) -> Vec<Entry> {
        self.layout()
            .0
            .into_iter()
            .filter(|entry| !entry.disabled.get_untracked())
            .collect()
    }

    fn entry(self, id: usize) -> Option<Entry> {
        self.try_command()?
            .entries
            .with_untracked(|entries| entries.iter().find(|entry| entry.id == id).copied())
    }

    // cmdk highlights the first item before anything is chosen, from the server's html on
    fn highlighted(self, entry: &Entry) -> bool {
        self.try_command().is_some_and(|command| {
            let value = command.value.with(|value| value.trim().to_string());
            if value.is_empty() {
                command.first.get() == Some(entry.id)
            } else {
                entry.value.with(|own| *own == value)
            }
        })
    }

    fn selected(self) -> Option<Entry> {
        let value = self.current();
        if value.is_empty() {
            return self.entry(self.try_command()?.first.get_untracked()?);
        }
        self.layout()
            .0
            .into_iter()
            .find(|entry| entry.value.with_untracked(|own| *own == value))
    }

    fn active_id(self) -> Option<String> {
        let command = self.try_command()?;
        let value = command.value.with(|value| value.trim().to_string());
        if value.is_empty() {
            let first = command.first.get()?;
            return Some(self.entry(first)?.element_id.get());
        }
        let matching: Vec<Entry> = command.entries.with(|entries| {
            entries
                .iter()
                .copied()
                .filter(|entry| entry.value.with(|own| *own == value) && self.shown(entry))
                .collect()
        });
        let entry = match matching.as_slice() {
            [] => return None,
            [entry] => *entry,
            _ => self
                .layout()
                .0
                .into_iter()
                .find(|entry| matching.iter().any(|own| own.id == entry.id))?,
        };
        Some(entry.element_id.get())
    }

    fn choose(self, value: String, reason: ChangeReason, reveal: bool) {
        let Some(command) = self.try_command() else {
            return;
        };
        if command.value.with_untracked(|own| *own == value) {
            return;
        }
        // a screen reader follows the list from the input
        if let Some(active) = document().active_element()
            && (active.has_attribute("cmdk-input") || active.has_attribute("cmdk-root"))
        {
            if let Some(input) = command.input.get_untracked() {
                let _ = input.focus();
            } else if let Some(list) = command.list.get_untracked() {
                let _ = list.focus();
            }
        }
        command.value.set(value, reason);
        if reveal {
            request_animation_frame(move || self.reveal());
        }
    }

    fn pick(self, entry: Entry) {
        let value = entry.value.get_untracked();
        self.choose(value.clone(), ChangeReason::ItemPress, false);
        if let Some(on_select) = entry.on_select {
            on_select.run(value);
        }
    }

    fn reveal(self) {
        let (items, groups) = self.layout();
        let Some(entry) = self.selected() else {
            return;
        };
        let Some(element) = entry.element.get_untracked() else {
            return;
        };
        if let Some(group) = entry.group
            && items
                .iter()
                .find(|own| own.group == Some(group))
                .is_some_and(|first| first.id == entry.id)
            && let Some(heading) = groups
                .iter()
                .find(|own| own.id == group)
                .and_then(|own| own.heading.get_untracked())
        {
            scroll_into_view(&heading);
        }
        scroll_into_view(&element);
    }

    fn select_first(self, reason: ChangeReason) {
        let value = self
            .valid()
            .first()
            .map(|entry| entry.value.get_untracked())
            .unwrap_or_default();
        self.choose(value, reason, true);
    }

    // after items mount: cmdk highlights the first when nothing is, and brings the highlight into view
    fn settle(self) {
        let Some(command) = self.try_command() else {
            return;
        };
        if command.settling.get_value() {
            return;
        }
        command.settling.set_value(true);
        request_animation_frame(move || {
            let Some(command) = self.try_command() else {
                return;
            };
            command.settling.set_value(false);
            if command.value.with_untracked(String::is_empty) {
                self.select_first(ChangeReason::None);
            } else {
                self.reveal();
            }
        });
    }

    fn select_at(self, index: usize) {
        if let Some(entry) = self.valid().get(index) {
            self.choose(
                entry.value.get_untracked(),
                ChangeReason::ListNavigation,
                true,
            );
        }
    }

    fn select_last(self) {
        if let Some(entry) = self.valid().last() {
            self.choose(
                entry.value.get_untracked(),
                ChangeReason::ListNavigation,
                true,
            );
        }
    }

    fn step(self, change: isize) {
        let Some(command) = self.try_command() else {
            return;
        };
        let items = self.valid();
        let selected = self.selected();
        let index = selected
            .and_then(|selected| items.iter().position(|entry| entry.id == selected.id))
            .and_then(|index| isize::try_from(index).ok())
            .unwrap_or(-1);
        let target = index + change;
        let next = if command.looping.get_untracked() {
            match usize::try_from(target) {
                Err(_) => items.last(),
                Ok(target) if target == items.len() => items.first(),
                Ok(target) => items.get(target),
            }
        } else {
            usize::try_from(target)
                .ok()
                .and_then(|target| items.get(target))
        };
        if let Some(entry) = next {
            self.choose(
                entry.value.get_untracked(),
                ChangeReason::ListNavigation,
                true,
            );
        }
    }

    fn step_group(self, change: isize) {
        let selected = self.selected();
        let Some(group) = selected.and_then(|selected| selected.group) else {
            self.step(change);
            return;
        };
        let (items, groups) = self.layout();
        let Some(at) = groups.iter().position(|own| own.id == group) else {
            self.step(change);
            return;
        };
        let siblings: Vec<&Group> = if change > 0 {
            groups[at + 1..].iter().collect()
        } else {
            groups[..at].iter().rev().collect()
        };
        let found = siblings.iter().find_map(|sibling| {
            items
                .iter()
                .find(|entry| entry.group == Some(sibling.id) && !entry.disabled.get_untracked())
        });
        match found {
            Some(entry) => {
                self.choose(
                    entry.value.get_untracked(),
                    ChangeReason::ListNavigation,
                    true,
                );
            }
            None => self.step(change),
        }
    }

    fn forward(self, event: &ev::KeyboardEvent, change: isize) {
        event.prevent_default();
        if event.meta_key() {
            if change > 0 {
                self.select_last();
            } else {
                self.select_at(0);
            }
        } else if event.alt_key() {
            self.step_group(change);
        } else {
            self.step(change);
        }
    }

    fn keydown(self, event: &ev::KeyboardEvent) {
        let Some(command) = self.try_command() else {
            return;
        };
        // an ime still composing keeps its keys
        if event.default_prevented() || event.is_composing() || event.key_code() == 229 {
            return;
        }
        let vim = command.vim_bindings.get_untracked() && event.ctrl_key();
        match event.key().as_str() {
            "n" | "j" if vim => self.forward(event, 1),
            "ArrowDown" => self.forward(event, 1),
            "p" | "k" if vim => self.forward(event, -1),
            "ArrowUp" => self.forward(event, -1),
            "Home" => {
                event.prevent_default();
                self.select_at(0);
            }
            "End" => {
                event.prevent_default();
                self.select_last();
            }
            "Enter" => {
                event.prevent_default();
                if let Some(entry) = self.selected()
                    && !entry.disabled.get_untracked()
                {
                    self.pick(entry);
                }
            }
            _ => {}
        }
    }
}

fn arrange(
    entries: Vec<Entry>,
    groups: Vec<Group>,
    filtering: bool,
    scores: &HashMap<usize, f64>,
    moved: &HashMap<usize, usize>,
) -> (Vec<Entry>, Vec<Group>) {
    let entries = in_order(entries, |entry| {
        entry.element.get_untracked().map(Into::into)
    });
    let groups = in_order(groups, |group| {
        group.element.get_untracked().map(Into::into)
    });
    let score = |id: usize| scores.get(&id).copied().unwrap_or_default();
    let shown: Vec<Entry> = entries
        .iter()
        .copied()
        .filter(|entry| !entry.counted || !filtering || score(entry.id) > 0.0)
        .collect();
    if !filtering {
        return (shown, groups);
    }
    let order = |id| moved.get(&id).copied().unwrap_or_default();
    let mut tops: Vec<(Top, HtmlElement)> = shown
        .iter()
        .filter(|entry| entry.group.is_none())
        .filter_map(|entry| Some((Top::Item(entry.id), entry.element.get_untracked()?.into())))
        .chain(groups.iter().filter_map(|group| {
            Some((Top::Group(group.id), group.element.get_untracked()?.into()))
        }))
        .collect();
    tops.sort_by(|(_, a), (_, b)| document_order(a, b));
    let places: HashMap<Top, usize> = tops
        .iter()
        .enumerate()
        .map(|(at, (key, _))| (*key, at))
        .collect();
    let place = |key| places.get(&key).copied().unwrap_or_default();
    let ranks: HashMap<usize, usize> = entries
        .iter()
        .enumerate()
        .map(|(at, entry)| (entry.id, at))
        .collect();
    let mut arranged = shown;
    arranged.sort_by_key(|entry| match entry.group {
        None => (order(entry.id), place(Top::Item(entry.id)), 0, 0),
        Some(group) => (
            0,
            place(Top::Group(group)),
            order(entry.id),
            ranks.get(&entry.id).copied().unwrap_or_default(),
        ),
    });
    (arranged, groups)
}

// in the document's order once they're all in it, the order they came in until then
fn in_order<T: Copy>(list: Vec<T>, element: impl Fn(&T) -> Option<HtmlElement>) -> Vec<T> {
    let placed: Option<Vec<(T, HtmlElement)>> = list
        .iter()
        .map(|item| Some((*item, element(item)?)))
        .collect();
    let Some(mut placed) = placed else {
        return list;
    };
    placed.sort_by(|(_, a), (_, b)| document_order(a, b));
    placed.into_iter().map(|(item, _)| item).collect()
}

fn scroll_into_view(element: &HtmlElement) {
    let options = ScrollIntoViewOptions::new();
    options.set_block(ScrollLogicalPosition::Nearest);
    element.scroll_into_view_with_scroll_into_view_options(&options);
}

// cmdk sorts by moving nodes, a column of flex items sorts by `order` and leaves leptos's nodes where they are
fn sorting(context: CommandContext) -> impl Fn() -> Option<&'static str> + Copy {
    move || {
        context
            .try_command()
            .is_some_and(|command| command.filtering.get())
            .then_some("flex")
    }
}

fn column(context: CommandContext) -> impl Fn() -> Option<&'static str> + Copy {
    move || sorting(context)().map(|_| "column")
}

#[component(element = "div")]
pub fn CommandRoot(
    #[prop(optional, into)] label: Option<Signal<String>>,
    #[prop(into, default = true.into())] should_filter: Signal<bool>,
    #[prop(optional, into)] filter: Option<CommandFilter>,
    #[prop(optional, into)] value: Option<Signal<String>>,
    #[prop(optional, into)] default_value: String,
    #[prop(optional, into)] on_value_change: Option<Callback<(String, ChangeReason)>>,
    #[prop(optional, into)] r#loop: Signal<bool>,
    #[prop(optional, into)] disable_pointer_selection: Signal<bool>,
    #[prop(into, default = true.into())] vim_bindings: Signal<bool>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let base = use_id();
    let label_ref = NodeRef::new();
    let input = NodeRef::new();
    let list = NodeRef::new();
    let search = RwSignal::new(None::<Controlled<String>>);
    let query = Signal::derive(move || search.get().map(|search| search.get()).unwrap_or_default());
    let filtering = Signal::derive(move || should_filter.get() && !query.with(String::is_empty));
    let entries = RwSignal::new(Vec::<Entry>::new());
    let groups = RwSignal::new(Vec::<Group>::new());
    let scores = Memo::new(move |_| {
        if !filtering.get() {
            return HashMap::new();
        }
        let query = query.get();
        entries.with(|entries| {
            entries
                .iter()
                .map(|entry| {
                    let value = entry.value.get();
                    let score = if value.is_empty() {
                        0.0
                    } else {
                        entry.keywords.with_value(|keywords| match filter {
                            Some(filter) => {
                                filter.run((value.clone(), query.clone(), keywords.clone()))
                            }
                            None => command_score(&value, &query, keywords),
                        })
                    };
                    (entry.id, score)
                })
                .collect()
        })
    });
    // cmdk moves highlightable items to the end of their group or list, best first; its group sort never matches
    let moved = Memo::new(move |_| {
        if !filtering.get() {
            return HashMap::new();
        }
        let scores = scores.get();
        let score = |id: usize| scores.get(&id).copied().unwrap_or_default();
        let items = in_order(entries.get(), |entry| {
            entry.element.get_untracked().map(Into::into)
        });
        let mut valid: Vec<Entry> = items
            .into_iter()
            .filter(|entry| (!entry.counted || score(entry.id) > 0.0) && !entry.disabled.get())
            .collect();
        valid.sort_by(|a, b| score(b.id).total_cmp(&score(a.id)));
        valid
            .iter()
            .enumerate()
            .map(|(at, entry)| (entry.id, at + 1))
            .collect()
    });
    let first = Memo::new(move |_| {
        arrange(
            entries.get(),
            groups.get(),
            filtering.get(),
            &scores.get(),
            &moved.get(),
        )
        .0
        .into_iter()
        .find(|entry| !entry.disabled.get())
        .map(|entry| entry.id)
    });
    let context = CommandContext(StoredValue::new(Command {
        input,
        list,
        label_id: ElementId::new(label_ref, format!("{base}-label")),
        input_id: ElementId::new(input, format!("{base}-input")),
        list_id: ElementId::new(list, format!("{base}-list")),
        search,
        query,
        filtering,
        value: Controlled::new(value, default_value, on_value_change),
        looping: r#loop,
        disable_pointer_selection,
        vim_bindings,
        entries,
        groups,
        next: StoredValue::new(0),
        scores,
        moved,
        first,
        settling: StoredValue::new(false),
    }));
    let command = context.command();
    view! {
        <div
            {..attrs}
            tabindex=move || tabindex.map_or(-1, |tabindex| tabindex.get())
            cmdk-root=""
            on:keydown=move |event: ev::KeyboardEvent| context.keydown(&event)
        >
            <label
                node_ref=label_ref
                cmdk-label=""
                for=move || command.input_id.get()
                id=move || command.label_id.get()
                style=SR_ONLY
            >
                {move || label.map(|label| label.get())}
            </label>
            <Provider value=context>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "input", skip(r#type, role, id, autocomplete))]
pub fn CommandInput(
    #[prop(optional, into)] value: Option<Signal<String>>,
    #[prop(optional, into)] on_value_change: Option<Callback<(String, ChangeReason)>>,
) -> impl IntoView {
    let context = expect_context::<CommandContext>();
    let command = context.command();
    let search = Controlled::new(value, String::new(), on_value_change);
    command.search.set(Some(search));
    view! {
        <input
            {..attrs}
            {..custom_attribute("autocorrect", "off")}
            node_ref=command.input
            cmdk-input=""
            autocomplete="off"
            spellcheck="false"
            aria-autocomplete="list"
            role="combobox"
            aria-expanded="true"
            aria-controls=move || command.list_id.get()
            aria-labelledby=move || command.label_id.get()
            aria-activedescendant=move || context.active_id()
            id=move || command.input_id.get()
            type="text"
            value=search.get_untracked()
            prop:value=move || search.get()
            on:input=move |event| {
                search.set(event_target_value(&event), ChangeReason::InputChange);
                context.select_first(ChangeReason::InputChange);
            }
        />
    }
}

#[component(element = "div", skip(role, tabindex, id))]
pub fn CommandList(
    #[prop(into, default = Signal::stored(String::from("Suggestions")))] label: Signal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<CommandContext>();
    let command = context.command();
    let sizer = NodeRef::<html::Div>::new();
    Effect::new(move |_| {
        let (Some(list), Some(inner)) = (command.list.get(), sizer.get()) else {
            return;
        };
        let measured = inner.clone();
        observe_resize(&inner, move || {
            let height = measured.offset_height();
            let _ = HtmlElement::style(&list)
                .set_property("--cmdk-list-height", &format!("{height}.0px"));
        });
    });
    view! {
        <div
            {..attrs}
            node_ref=command.list
            cmdk-list=""
            role="listbox"
            tabindex="-1"
            aria-activedescendant=move || context.active_id()
            aria-label=move || label.get()
            id=move || command.list_id.get()
        >
            <div
                node_ref=sizer
                cmdk-list-sizer=""
                style:display=sorting(context)
                style:flex-direction=column(context)
            >
                {children.map(|children| children())}
            </div>
        </div>
    }
}

#[component(element = "div", skip(id, role))]
pub fn CommandItem(
    #[prop(optional, into)] value: Option<String>,
    #[prop(optional)] keywords: Vec<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] on_select: Option<Callback<String>>,
    #[prop(optional)] force_mount: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<CommandContext>();
    let command = context.command();
    let scope = use_context::<GroupScope>();
    let element = NodeRef::new();
    let given = value.map(|value| value.trim().to_string());
    let value = RwSignal::new(given.clone().unwrap_or_default());
    let entry = Entry {
        id: context.next_id(),
        element,
        element_id: ElementId::new(element, use_id()),
        value,
        keywords: StoredValue::new(
            keywords
                .into_iter()
                .map(|keyword| keyword.trim().to_string())
                .collect(),
        ),
        group: scope.map(|scope| scope.id),
        counted: !force_mount && !scope.is_some_and(|scope| scope.forced),
        disabled,
        on_select,
    };
    let id = entry.id;
    command.entries.update(|entries| entries.push(entry));
    on_cleanup(move || {
        let current = context.current();
        let selected =
            !current.is_empty() && value.try_with_untracked(|own| *own == current) == Some(true);
        if let Some(command) = context.try_command() {
            command
                .entries
                .try_update(|entries| entries.retain(|entry| entry.id != id));
        }
        if selected {
            request_animation_frame(move || {
                if context.try_command().is_some() {
                    context.select_first(ChangeReason::None);
                }
            });
        }
    });
    // cmdk's value is the item's text when it isn't given one
    Effect::new(move |_| {
        let Some(node) = element.get() else {
            return;
        };
        if given.is_none() {
            let text = node.text_content().unwrap_or_default();
            value.set(text.trim().to_string());
        }
        context.settle();
    });
    let selected = move || context.highlighted(&entry);
    view! {
        <div
            {..attrs}
            node_ref=element
            id=move || entry.element_id.get()
            cmdk-item=""
            role="option"
            aria-disabled=move || disabled.get().to_string()
            aria-selected=move || selected().to_string()
            data-disabled=move || disabled.get().to_string()
            data-selected=move || selected().to_string()
            data-value=move || value.with(|value| (!value.is_empty()).then(|| value.clone()))
            hidden=move || !context.shown(&entry)
            style:order=move || context.order(id)
            on:pointermove=move |_| {
                if !disabled.get_untracked() && !command.disable_pointer_selection.get_untracked() {
                    context.choose(value.get_untracked(), ChangeReason::Pointer, false);
                }
            }
            on:click=move |_| {
                if !disabled.get_untracked() {
                    context.pick(entry);
                }
            }
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div", skip(role))]
pub fn CommandGroup(
    #[prop(optional, into)] heading: Option<String>,
    #[prop(optional, into)] value: Option<String>,
    #[prop(optional)] force_mount: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<CommandContext>();
    let command = context.command();
    let id = context.next_id();
    let element = NodeRef::new();
    let heading_ref = NodeRef::new();
    let heading_id = ElementId::new(heading_ref, use_id());
    command.groups.update(|groups| {
        groups.push(Group {
            id,
            element,
            heading: heading_ref,
        });
    });
    on_cleanup(move || {
        if let Some(command) = context.try_command() {
            command
                .groups
                .try_update(|groups| groups.retain(|group| group.id != id));
        }
    });
    let titled = heading.is_some();
    let named = value
        .or_else(|| heading.clone())
        .map(|value| value.trim().to_string());
    view! {
        <div
            {..attrs}
            node_ref=element
            cmdk-group=""
            role="presentation"
            hidden=move || !force_mount && !context.group_shown(id)
            data-value=named
        >
            {heading.map(|heading| {
                view! {
                    <div
                        node_ref=heading_ref
                        cmdk-group-heading=""
                        aria-hidden="true"
                        id=move || heading_id.get()
                    >
                        {heading}
                    </div>
                }
            })}
            <div
                cmdk-group-items=""
                role="group"
                aria-labelledby=move || titled.then(|| heading_id.get())
                style:display=sorting(context)
                style:flex-direction=column(context)
            >
                <Provider value=GroupScope { id, forced: force_mount }>
                    {children.map(|children| children())}
                </Provider>
            </div>
        </div>
    }
}

#[component(element = "div", skip(role))]
pub fn CommandSeparator(#[prop(optional)] always_render: bool) -> impl IntoView {
    let command = expect_context::<CommandContext>().command();
    view! {
        <div
            {..attrs}
            cmdk-separator=""
            role="separator"
            hidden=move || !always_render && !command.query.with(String::is_empty)
        />
    }
}

#[component(element = "div", skip(role))]
pub fn CommandEmpty(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<CommandContext>();
    view! {
        <div {..attrs} cmdk-empty="" role="presentation" hidden=move || context.count() != 0>
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div", skip(role))]
pub fn CommandLoading(
    #[prop(optional, into)] progress: Option<Signal<f64>>,
    #[prop(into, default = Signal::stored(String::from("Loading...")))] label: Signal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div
            {..attrs}
            cmdk-loading=""
            role="progressbar"
            aria-valuenow=move || progress.map(|progress| progress.get())
            aria-valuemin="0"
            aria-valuemax="100"
            aria-label=move || label.get()
        >
            <div aria-hidden="true">{children.map(|children| children())}</div>
        </div>
    }
}
