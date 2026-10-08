
pub struct Component {
    pub name: &'static str,
    pub source: &'static str,
    pub dependencies: &'static [&'static str],
}

pub const COMPONENTS: &[Component] = &[
    Component {
        name: "button",
        source: include_str!("../../leptos-kit/src/button.rs"),
        dependencies: &[],
    },
    Component {
        name: "dialog",
        source: include_str!("../../leptos-kit/src/dialog.rs"),
        dependencies: &["button"],
    },
    Component {
        name: "popover",
        source: include_str!("../../leptos-kit/src/popover.rs"),
        dependencies: &["button"],
    },
];

pub const UTILS: &str = include_str!("../../leptos-kit/src/utils.rs");

pub const STYLES: &[(&str, &str)] = &[
    (
        "vega",
        include_str!("../../leptos-kit/styles/style-vega.css"),
    ),
    (
        "nova",
        include_str!("../../leptos-kit/styles/style-nova.css"),
    ),
    (
        "maia",
        include_str!("../../leptos-kit/styles/style-maia.css"),
    ),
    (
        "lyra",
        include_str!("../../leptos-kit/styles/style-lyra.css"),
    ),
    (
        "mira",
        include_str!("../../leptos-kit/styles/style-mira.css"),
    ),
    (
        "luma",
        include_str!("../../leptos-kit/styles/style-luma.css"),
    ),
    (
        "sera",
        include_str!("../../leptos-kit/styles/style-sera.css"),
    ),
    (
        "rhea",
        include_str!("../../leptos-kit/styles/style-rhea.css"),
    ),
];

pub const BASE_COLORS: &[(&str, &str)] = &[
    (
        "neutral",
        include_str!("../../leptos-kit/styles/themes/neutral.css"),
    ),
    (
        "stone",
        include_str!("../../leptos-kit/styles/themes/stone.css"),
    ),
    (
        "zinc",
        include_str!("../../leptos-kit/styles/themes/zinc.css"),
    ),
    (
        "mauve",
        include_str!("../../leptos-kit/styles/themes/mauve.css"),
    ),
    (
        "olive",
        include_str!("../../leptos-kit/styles/themes/olive.css"),
    ),
    (
        "mist",
        include_str!("../../leptos-kit/styles/themes/mist.css"),
    ),
    (
        "taupe",
        include_str!("../../leptos-kit/styles/themes/taupe.css"),
    ),
];

pub const STRUCTURE: &str = include_str!("../../leptos-kit/styles/structure.css");

pub const SHARED: &[(&str, &str)] = &[
    (
        "tw-animate.css",
        include_str!("../../leptos-kit/styles/tw-animate.css"),
    ),
    (
        "shadcn-tailwind.css",
        include_str!("../../leptos-kit/styles/shadcn-tailwind.css"),
    ),
    (
        "theme.css",
        include_str!("../../leptos-kit/styles/theme.css"),
    ),
    (
        "leptos-kit.css",
        include_str!("../../leptos-kit/styles/leptos-kit.css"),
    ),
];

pub const ICONS: &str = include_str!("../../leptos-kit-icons/src/semantic.rs");
pub const ICON_LIBRARIES: &[&str] = &["lucide"];

pub fn component(name: &str) -> Option<&'static Component> {
    COMPONENTS.iter().find(|component| component.name == name)
}
