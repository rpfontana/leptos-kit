// svg files rather than `icon-nodes.json`

use super::{Data, Files, Icon, Library, Style, svg_body};
use crate::Result;

pub static LIBRARY: Library = Library {
    name: "lucide",
    react_prefix: "",
    react_suffix: "Icon",
    package: "lucide-static",
    version: "1.48.0",
    data: Data::Dir {
        path: "icons/",
        extension: ".svg",
    },
    licence_path: "LICENSE",
    licence: "ISC",
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

fn parse(files: &Files) -> Result<Vec<Icon>> {
    files
        .iter()
        .map(|(name, contents)| {
            Ok(Icon {
                name: name.clone(),
                body: svg_body(name, contents, &LIBRARY.style)?,
            })
        })
        .collect()
}
