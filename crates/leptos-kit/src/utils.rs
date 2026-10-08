use leptos::prelude::*;

pub(crate) fn classes(
    own: impl Into<String>,
    class: Option<Signal<String>>,
) -> impl Fn() -> String {
    let own = own.into();
    move || match class {
        Some(class) => format!("{own} {}", class.get()),
        None => own.clone(),
    }
}
