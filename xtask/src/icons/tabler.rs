use super::{Data, Icon, Library, Style, svg_body};
use crate::Result;

pub static LIBRARY: Library = Library {
    name: "tabler",
    react_prefix: "Icon",
    react_suffix: "",
    package: "@tabler/icons",
    version: "3.49.0",
    data: Data::Dir {
        path: "icons/outline/",
        extension: ".svg",
    },
    licence_path: "LICENSE",
    licence: "MIT",
    style: Style {
        view_box: "0 0 24 24",
        fill: "none",
        stroke: "currentColor",
        stroke_width: "2",
        stroke_linecap: "round",
        stroke_linejoin: "round",
    },
    parse,
};

// an invisible box at the top of every file, the react package drops it
const BOX: &str = r#"<path stroke="none" d="M0 0h24v24H0z" fill="none"/>"#;

fn parse(files: &[(String, String)]) -> Result<Vec<Icon>> {
    files
        .iter()
        .map(|(name, contents)| {
            let body = svg_body(name, contents, &LIBRARY.style)?;
            Ok(Icon {
                name: name.clone(),
                body: body.replace(BOX, "").trim().to_string(),
            })
        })
        .collect()
}
