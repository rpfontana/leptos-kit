mod button;
mod dialog;
mod popover;

pub use button::Button;
pub use dialog::{
    Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader,
    DialogOverlay, DialogPortal, DialogTitle, DialogTrigger,
};
pub use popover::{
    Popover, PopoverContent, PopoverDescription, PopoverHeader, PopoverTitle, PopoverTrigger,
};

use leptos::prelude::*;

fn classes(own: impl Into<String>, class: Option<Signal<String>>) -> impl Fn() -> String {
    let own = own.into();
    move || match class {
        Some(class) => format!("{own} {}", class.get()),
        None => own.clone(),
    }
}
