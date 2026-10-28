use leptos::ev;
use leptos::prelude::*;
use std::time::Duration;

pub(crate) enum Typed<T> {
    // the key doesn't type
    Ignored,
    Missed,
    Matched(T),
}

impl<T> Typed<T> {
    pub(crate) fn map<U>(self, f: impl FnOnce(T) -> U) -> Typed<U> {
        match self {
            Typed::Ignored => Typed::Ignored,
            Typed::Missed => Typed::Missed,
            Typed::Matched(found) => Typed::Matched(f(found)),
        }
    }
}

// base ui's typeahead: the typed letters pick the next item whose label starts with them
#[derive(Clone, Copy)]
pub(crate) struct Typeahead {
    text: StoredValue<String>,
    from: StoredValue<Option<usize>>,
    matched: StoredValue<Option<usize>>,
    reset: StoredValue<Option<TimeoutHandle>>,
    wait: u32,
}

impl Typeahead {
    pub(crate) fn new(wait: u32) -> Self {
        let this = Self {
            text: StoredValue::new(String::new()),
            from: StoredValue::new(None),
            matched: StoredValue::new(None),
            reset: StoredValue::new(None),
            wait,
        };
        on_cleanup(move || this.stop());
        this
    }

    pub(crate) fn typing(self) -> bool {
        self.text
            .try_with_value(|text| !text.is_empty())
            .unwrap_or(false)
    }

    pub(crate) fn clear(self) {
        self.stop();
        self.text.try_set_value(String::new());
        self.matched.try_set_value(None);
    }

    fn stop(self) {
        if let Some(Some(handle)) = self.reset.try_update_value(Option::take) {
            handle.clear();
        }
    }

    // an item without a label can't be picked
    pub(crate) fn type_key(
        self,
        event: &ev::KeyboardEvent,
        labels: &[Option<String>],
        at: Option<usize>,
    ) -> Typed<usize> {
        let key = event.key();
        let typing = self.typing();
        if key.chars().count() != 1 || event.ctrl_key() || event.meta_key() || event.alt_key() {
            return Typed::Ignored;
        }
        if (key == " " && !typing) || labels.is_empty() {
            return Typed::Ignored;
        }
        let labels: Vec<Option<String>> = labels
            .iter()
            .map(|label| label.as_ref().map(|label| label.to_lowercase()))
            .collect();
        if !typing {
            self.from.set_value(at);
        }
        let key = key.to_lowercase();
        // repeating one letter cycles through the items that start with it
        let cycles = labels.iter().flatten().all(|label| {
            let mut chars = label.chars();
            match (chars.next(), chars.next()) {
                (Some(first), Some(second)) => first != second,
                _ => true,
            }
        });
        if cycles && self.text.get_value() == key {
            self.text.set_value(String::new());
            self.from.set_value(self.matched.get_value());
        }
        self.text.update_value(|text| text.push_str(&key));
        self.stop();
        let reset = set_timeout_with_handle(
            move || {
                self.text.try_set_value(String::new());
                if let Some(matched) = self.matched.try_get_value() {
                    self.from.try_set_value(matched);
                }
            },
            Duration::from_millis(self.wait.into()),
        );
        self.reset.set_value(reset.ok());
        let text = self.text.get_value();
        let start = self.from.get_value().map_or(0, |from| from + 1);
        let found = (0..labels.len())
            .map(|offset| (start + offset) % labels.len())
            .find(|&index| {
                labels[index]
                    .as_ref()
                    .is_some_and(|label| label.starts_with(&text))
            });
        if let Some(index) = found {
            self.matched.set_value(Some(index));
            return Typed::Matched(index);
        }
        if key != " " {
            self.text.set_value(String::new());
        }
        Typed::Missed
    }
}
