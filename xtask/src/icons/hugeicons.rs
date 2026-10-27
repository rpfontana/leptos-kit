// names from `index.js`, since shadcn imports aliases (`SearchIcon` is `Search01Icon`)

use super::{Data, Files, Icon, Library, Style, upstream_name};
use crate::Result;
use std::collections::BTreeMap;
use std::fmt::Write as _;

pub static LIBRARY: Library = Library {
    name: "hugeicons",
    react_prefix: "",
    react_suffix: "Icon",
    package: "@hugeicons/core-free-icons",
    version: "4.3.5",
    data: Data::Dir {
        path: "dist/esm/",
        extension: ".js",
    },
    licence_path: "LICENSE.md",
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

fn parse(files: &Files) -> Result<Vec<Icon>> {
    let files: BTreeMap<&str, &str> = files
        .iter()
        .map(|(name, contents)| (name.as_str(), contents.as_str()))
        .collect();
    let index = files.get("index").ok_or("no index.js")?;

    let mut names: BTreeMap<String, &str> = BTreeMap::new();
    for line in index.lines().filter(|line| line.starts_with("export ")) {
        let (exports, file) = line
            .strip_prefix("export { ")
            .and_then(|rest| rest.strip_suffix(".js';"))
            .and_then(|rest| rest.split_once(" } from './"))
            .ok_or_else(|| format!("index.js: unexpected line {line}"))?;
        for export in exports.split(", ") {
            let export = export
                .strip_prefix("default as ")
                .ok_or_else(|| format!("index.js: unexpected export {export}"))?;
            if !export.ends_with("Icon") {
                continue;
            }
            let name = upstream_name(export, &LIBRARY);
            if let Some(other) = names.insert(name.clone(), file)
                && other != file
            {
                return Err(format!("`{name}` is both {other}.js and {file}.js").into());
            }
        }
    }

    names
        .into_iter()
        .map(|(name, file)| {
            let source = files
                .get(file)
                .ok_or_else(|| format!("index.js points at a missing {file}.js"))?;
            Ok(Icon {
                body: body(&name, source)?,
                name,
            })
        })
        .collect()
}

fn body(name: &str, source: &str) -> Result<String> {
    let mut elements = Vec::new();
    for line in source.lines().map(str::trim) {
        if !line.starts_with('[') {
            continue;
        }
        let (tag, attributes) = line
            .trim_end_matches(',')
            .strip_prefix("[\"")
            .and_then(|rest| rest.strip_suffix(" }]"))
            .and_then(|rest| rest.split_once("\", { "))
            .ok_or_else(|| format!("{name}: unexpected element {line}"))?;

        let mut attributes = attributes
            .split("\", ")
            .map(|pair| {
                let (key, value) = pair
                    .split_once(": \"")
                    .ok_or_else(|| format!("{name}: unexpected attribute {pair}"))?;
                Ok((kebab_case(key), value.trim_end_matches('"')))
            })
            .collect::<Result<Vec<_>>>()?;

        // shadcn passes `strokeWidth={2}`, which `HugeiconsIcon` puts on every element
        attributes.retain(|(key, _)| !matches!(key.as_str(), "key" | "stroke" | "stroke-width"));
        // `HugeiconsIcon` leaves these unset on the svg
        for (attribute, initial) in [("stroke-linecap", "butt"), ("stroke-linejoin", "miter")] {
            match attributes.iter().position(|(key, _)| key == attribute) {
                Some(index) if attributes[index].1 == "round" => {
                    attributes.remove(index);
                }
                Some(_) => {}
                None => attributes.push((attribute.to_string(), initial)),
            }
        }
        elements.push((tag, attributes));
    }
    if elements.is_empty() {
        return Err(format!("{name}: empty icon").into());
    }

    // `HugeiconsIcon` draws the translucent elements first
    elements.sort_by_key(|(_, attributes)| !attributes.iter().any(|(key, _)| key == "opacity"));
    let mut out = String::new();
    for (tag, attributes) in &elements {
        write!(out, "<{tag}")?;
        for (key, value) in attributes {
            write!(out, " {key}=\"{value}\"")?;
        }
        out.push_str("/>");
    }
    Ok(out)
}

fn kebab_case(name: &str) -> String {
    let mut out = String::new();
    for char in name.chars() {
        if char.is_ascii_uppercase() {
            out.push('-');
        }
        out.push(char.to_ascii_lowercase());
    }
    out
}
