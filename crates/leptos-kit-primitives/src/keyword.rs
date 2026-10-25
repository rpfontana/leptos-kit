use leptos::prelude::*;

pub trait Keyword: Copy + Send + Sync + 'static {
    fn as_str(self) -> &'static str;
}

// generic, so one attribute closure per keyword type rather than per call site
pub fn keyword<K: Keyword>(
    value: Signal<K>,
) -> impl Fn() -> &'static str + Copy + Send + Sync + 'static {
    move || value.get().as_str()
}

#[macro_export]
macro_rules! keywords {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident = $text:literal),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        $vis enum $name {
            $($(#[$variant_meta])* $variant),*
        }

        impl $name {
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $text),*
                }
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl $crate::Keyword for $name {
            fn as_str(self) -> &'static str {
                $name::as_str(self)
            }
        }
    };
}

keywords! {
    #[derive(Default)]
    pub enum Orientation {
        #[default]
        Horizontal = "horizontal",
        Vertical = "vertical",
    }
}

keywords! {
    #[derive(Default)]
    pub enum Direction {
        #[default]
        Ltr = "ltr",
        Rtl = "rtl",
    }
}

keywords! {
    #[derive(Default)]
    pub enum Side {
        Top = "top",
        #[default]
        Bottom = "bottom",
        Left = "left",
        Right = "right",
        InlineStart = "inline-start",
        InlineEnd = "inline-end",
    }
}

keywords! {
    #[derive(Default)]
    pub enum Align {
        Start = "start",
        #[default]
        Center = "center",
        End = "end",
    }
}

keywords! {
    #[derive(Default)]
    pub(crate) enum TransitionStatus {
        #[default]
        Idle = "idle",
        Starting = "starting",
        Ending = "ending",
    }
}
