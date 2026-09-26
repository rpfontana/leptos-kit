//! the icons come from the package's SVG files rather than its
//! `icon-nodes.json`, because that file holds canonical names only and shadcn
//! refers to aliases too (`more-horizontal` is an alias of `ellipsis`)

use super::{Data, Icon, Library, Style};
use crate::Result;

pub static LIBRARY: Library = Library {
    name: "lucide",
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

fn parse(files: &[(String, String)]) -> Result<Vec<Icon>> {
    files
        .iter()
        .map(|(name, contents)| {
            Ok(Icon {
                name: name.clone(),
                body: body(name, contents)?,
            })
        })
        .collect()
}

/// fails if upstream's `<svg>` attributes are no longer the ones [`LIBRARY`]
/// records, rather than generating icons that render wrong
fn body(name: &str, svg: &str) -> Result<String> {
    // the files start with a licence comment, so look for `<svg` rather than for
    // the first `>`
    let open_start = svg
        .find("<svg")
        .ok_or_else(|| format!("{name}: no <svg> tag"))?;
    let open_end = open_start
        + svg[open_start..]
            .find('>')
            .ok_or_else(|| format!("{name}: unterminated <svg> tag"))?;
    let close = svg
        .rfind("</svg>")
        .ok_or_else(|| format!("{name}: no closing tag"))?;
    let (open, inner) = (&svg[open_start..open_end], &svg[open_end + 1..close]);

    for (attribute, value) in [
        ("viewBox", LIBRARY.style.view_box),
        ("fill", LIBRARY.style.fill),
        ("stroke", LIBRARY.style.stroke),
        ("stroke-width", LIBRARY.style.stroke_width),
        ("stroke-linecap", LIBRARY.style.stroke_linecap),
        ("stroke-linejoin", LIBRARY.style.stroke_linejoin),
    ] {
        let expected = format!("{attribute}=\"{value}\"");
        if !open.contains(&expected) {
            return Err(format!("{name}: expected {expected} in the <svg> tag").into());
        }
    }

    let body = inner
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" />", "/>");
    if body.is_empty() {
        return Err(format!("{name}: empty icon").into());
    }
    Ok(body)
}
