use leptos::prelude::*;

pub(crate) trait Own: Send + Sync + 'static {
    fn own(&self) -> String;
}

impl Own for &'static str {
    fn own(&self) -> String {
        self.to_string()
    }
}

impl<F: Fn() -> String + Send + Sync + 'static> Own for F {
    fn own(&self) -> String {
        self()
    }
}

pub(crate) fn classes(own: impl Own, class: Option<Signal<String>>) -> impl Fn() -> String {
    move || match class {
        Some(class) => format!("{} {}", own.own(), class.get()),
        None => own.own(),
    }
}
