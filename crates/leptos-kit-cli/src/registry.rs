use crate::Result;

pub struct Component {
    pub name: &'static str,
    pub source: &'static str,
    pub dependencies: &'static [&'static str],
    pub css: Option<&'static str>,
}

pub const COMPONENTS: &[Component] = &[
    Component {
        name: "badge",
        source: include_str!("../../leptos-kit/src/badge.rs"),
        dependencies: &[],
        css: None,
    },
    Component {
        name: "button",
        source: include_str!("../../leptos-kit/src/button.rs"),
        dependencies: &[],
        css: None,
    },
    Component {
        name: "card",
        source: include_str!("../../leptos-kit/src/card.rs"),
        dependencies: &[],
        css: None,
    },
    Component {
        name: "dialog",
        source: include_str!("../../leptos-kit/src/dialog.rs"),
        dependencies: &["button"],
        css: Some(include_str!(
            "../../leptos-kit/styles/components/dialog.css"
        )),
    },
    Component {
        name: "kbd",
        source: include_str!("../../leptos-kit/src/kbd.rs"),
        dependencies: &[],
        css: None,
    },
    Component {
        name: "label",
        source: include_str!("../../leptos-kit/src/label.rs"),
        dependencies: &[],
        css: None,
    },
    Component {
        name: "popover",
        source: include_str!("../../leptos-kit/src/popover.rs"),
        dependencies: &["button"],
        css: None,
    },
    Component {
        name: "separator",
        source: include_str!("../../leptos-kit/src/separator.rs"),
        dependencies: &[],
        css: Some(include_str!(
            "../../leptos-kit/styles/components/separator.css"
        )),
    },
    Component {
        name: "skeleton",
        source: include_str!("../../leptos-kit/src/skeleton.rs"),
        dependencies: &[],
        css: None,
    },
    Component {
        name: "spinner",
        source: include_str!("../../leptos-kit/src/spinner.rs"),
        dependencies: &[],
        css: Some(include_str!(
            "../../leptos-kit/styles/components/spinner.css"
        )),
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
];

pub const THEME: &str = include_str!("../../leptos-kit/styles/theme.css");

pub const ICONS: &str = include_str!("../../leptos-kit-icons/src/semantic.rs");
pub const ICON_LIBRARIES: &[&str] = &["lucide", "tabler", "hugeicons"];

pub fn pick(list: &'static [(&str, &str)], name: &str, what: &str) -> Result<&'static str> {
    list.iter()
        .find(|(known, _)| *known == name)
        .map(|(_, value)| *value)
        .ok_or_else(|| {
            let known: Vec<&str> = list.iter().map(|(known, _)| *known).collect();
            format!("unknown {what} `{name}`, expected one of {known:?}").into()
        })
}

pub fn resolve(name: &str, order: &mut Vec<&'static Component>) -> Result {
    let component = COMPONENTS
        .iter()
        .find(|component| component.name == name)
        .ok_or_else(|| format!("no component `{name}`, `leptos-kit list` shows them"))?;
    if order.iter().any(|done| done.name == name) {
        return Ok(());
    }
    for dependency in component.dependencies {
        resolve(dependency, order)?;
    }
    order.push(component);
    Ok(())
}
