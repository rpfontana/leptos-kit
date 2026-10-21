// the regular weight, which shadcn uses

use super::{Data, Icon, Library, Style, svg_body};
use crate::Result;

pub static LIBRARY: Library = Library {
    name: "phosphor",
    react_prefix: "",
    react_suffix: "Icon",
    package: "@phosphor-icons/core",
    version: "2.1.1",
    data: Data::Dir {
        path: "assets/regular/",
        extension: ".svg",
    },
    licence_path: "LICENSE",
    licence: "MIT",
    // filled, the stroke is left at its initial values
    style: Style {
        view_box: "0 0 256 256",
        fill: "currentColor",
        stroke: "none",
        stroke_width: "1",
        stroke_linecap: "butt",
        stroke_linejoin: "miter",
    },
    parse,
};

fn parse(files: &[(String, String)]) -> Result<Vec<Icon>> {
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
