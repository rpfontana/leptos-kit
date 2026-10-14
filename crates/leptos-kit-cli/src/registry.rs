use crate::Result;

pub struct Component {
    pub name: &'static str,
    pub source: &'static str,
    pub dependencies: &'static [&'static str],
    pub css: Option<&'static str>,
}

impl Component {
    pub fn module(&self) -> String {
        self.name.replace('-', "_")
    }
}

pub const COMPONENTS: &[Component] = &[
    Component {
        name: "alert",
        source: include_str!("../../leptos-kit/src/alert.rs"),
        dependencies: &[],
        css: None,
    },
    Component {
        name: "aspect-ratio",
        source: include_str!("../../leptos-kit/src/aspect_ratio.rs"),
        dependencies: &[],
        css: Some(include_str!(
            "../../leptos-kit/styles/components/aspect-ratio.css"
        )),
    },
    Component {
        name: "attachment",
        source: include_str!("../../leptos-kit/src/attachment.rs"),
        dependencies: &["button"],
        css: Some(include_str!(
            "../../leptos-kit/styles/components/attachment.css"
        )),
    },
    Component {
        name: "badge",
        source: include_str!("../../leptos-kit/src/badge.rs"),
        dependencies: &[],
        css: None,
    },
    Component {
        name: "breadcrumb",
        source: include_str!("../../leptos-kit/src/breadcrumb.rs"),
        dependencies: &[],
        css: None,
    },
    Component {
        name: "bubble",
        source: include_str!("../../leptos-kit/src/bubble.rs"),
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
        name: "button-group",
        source: include_str!("../../leptos-kit/src/button_group.rs"),
        dependencies: &[],
        css: Some(include_str!(
            "../../leptos-kit/styles/components/button-group.css"
        )),
    },
    Component {
        name: "card",
        source: include_str!("../../leptos-kit/src/card.rs"),
        dependencies: &[],
        css: None,
    },
    Component {
        name: "checkbox",
        source: include_str!("../../leptos-kit/src/checkbox.rs"),
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
        name: "empty",
        source: include_str!("../../leptos-kit/src/empty.rs"),
        dependencies: &[],
        css: None,
    },
    Component {
        name: "field",
        source: include_str!("../../leptos-kit/src/field.rs"),
        dependencies: &["separator"],
        css: None,
    },
    Component {
        name: "input",
        source: include_str!("../../leptos-kit/src/input.rs"),
        dependencies: &[],
        css: None,
    },
    Component {
        name: "input-group",
        source: include_str!("../../leptos-kit/src/input_group.rs"),
        dependencies: &["button", "input", "textarea"],
        css: None,
    },
    Component {
        name: "item",
        source: include_str!("../../leptos-kit/src/item.rs"),
        dependencies: &[],
        css: Some(include_str!("../../leptos-kit/styles/components/item.css")),
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
        name: "marker",
        source: include_str!("../../leptos-kit/src/marker.rs"),
        dependencies: &[],
        css: None,
    },
    Component {
        name: "message",
        source: include_str!("../../leptos-kit/src/message.rs"),
        dependencies: &[],
        css: None,
    },
    Component {
        name: "native-select",
        source: include_str!("../../leptos-kit/src/native_select.rs"),
        dependencies: &[],
        css: Some(include_str!(
            "../../leptos-kit/styles/components/native-select.css"
        )),
    },
    Component {
        name: "pagination",
        source: include_str!("../../leptos-kit/src/pagination.rs"),
        dependencies: &["button"],
        css: None,
    },
    Component {
        name: "popover",
        source: include_str!("../../leptos-kit/src/popover.rs"),
        dependencies: &["button"],
        css: None,
    },
    Component {
        name: "progress",
        source: include_str!("../../leptos-kit/src/progress.rs"),
        dependencies: &[],
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
    Component {
        name: "switch",
        source: include_str!("../../leptos-kit/src/switch.rs"),
        dependencies: &[],
        css: None,
    },
    Component {
        name: "table",
        source: include_str!("../../leptos-kit/src/table.rs"),
        dependencies: &[],
        css: None,
    },
    Component {
        name: "textarea",
        source: include_str!("../../leptos-kit/src/textarea.rs"),
        dependencies: &[],
        css: None,
    },
    Component {
        name: "toggle",
        source: include_str!("../../leptos-kit/src/toggle.rs"),
        dependencies: &[],
        css: None,
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
