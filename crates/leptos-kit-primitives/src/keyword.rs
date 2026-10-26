use leptos::prelude::*;
use leptos_kit_macros::Keyword;

// public for the styled parts the cli copies into apps, not for callers
#[doc(hidden)]
pub trait Keyword: Copy + Send + Sync + 'static {
    fn as_str(self) -> &'static str;
}

// generic, so one attribute closure per keyword type rather than per call site
#[doc(hidden)]
pub fn keyword<K: Keyword>(
    value: Signal<K>,
) -> impl Fn() -> &'static str + Copy + Send + Sync + 'static {
    move || value.get().as_str()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Keyword)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Keyword)]
pub enum Direction {
    #[default]
    Ltr,
    Rtl,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Keyword)]
pub enum Side {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
    InlineStart,
    InlineEnd,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Keyword)]
pub enum Align {
    Start,
    #[default]
    Center,
    End,
}
