use crate::Result;
use crate::registry::{BASE_COLORS, Component, ICON_LIBRARIES, STYLES, pick, resolve};
use std::fs;
use std::path::PathBuf;
use toml_edit::{DocumentMut, Item};

pub const CONFIG: &str = "leptos-kit.toml";

pub struct Config {
    pub styles: Vec<String>,
    pub base_colors: Vec<String>,
    pub icons: String,
    pub components: Vec<String>,
    pub components_dir: PathBuf,
    pub css_dir: PathBuf,
}

impl Config {
    pub fn from(doc: &DocumentMut) -> Result<Self> {
        let list = |key: &str| -> Result<Vec<String>> {
            let array = doc
                .get(key)
                .and_then(Item::as_array)
                .ok_or_else(|| format!("{CONFIG} has no `{key}` list"))?;
            Ok(array
                .iter()
                .filter_map(|value| value.as_str().map(String::from))
                .collect())
        };
        let path = |key: &str| -> Result<PathBuf> {
            doc.get("paths")
                .and_then(|paths| paths.get(key))
                .and_then(Item::as_str)
                .map(PathBuf::from)
                .ok_or_else(|| format!("{CONFIG} has no `paths.{key}`").into())
        };
        let config = Self {
            styles: list("styles")?,
            base_colors: list("base-colors")?,
            icons: doc
                .get("icons")
                .and_then(Item::as_str)
                .ok_or_else(|| format!("{CONFIG} has no `icons`"))?
                .to_string(),
            components: list("components")?,
            components_dir: path("components")?,
            css_dir: path("css")?,
        };
        config.check()?;
        Ok(config)
    }

    fn check(&self) -> Result {
        if self.styles.is_empty() || self.base_colors.is_empty() {
            return Err("at least one style and one base colour".into());
        }
        for style in &self.styles {
            pick(STYLES, style, "style")?;
        }
        for color in &self.base_colors {
            pick(BASE_COLORS, color, "base colour")?;
        }
        if !ICON_LIBRARIES.contains(&self.icons.as_str()) {
            return Err(format!(
                "unknown icon library `{}`, expected one of {ICON_LIBRARIES:?}",
                self.icons
            )
            .into());
        }
        self.order()?;
        Ok(())
    }

    // dependencies first
    pub fn order(&self) -> Result<Vec<&'static Component>> {
        let mut order = Vec::new();
        for name in &self.components {
            resolve(name, &mut order)?;
        }
        Ok(order)
    }

    pub fn sheets(&self) -> Vec<(&str, &'static str)> {
        self.styles
            .iter()
            .filter_map(|name| Some((name.as_str(), pick(STYLES, name, "style").ok()?)))
            .collect()
    }
}

pub fn load() -> Result<DocumentMut> {
    let text = fs::read_to_string(CONFIG)
        .map_err(|_| format!("no {CONFIG} here, run `leptos-kit init` first"))?;
    Ok(text
        .parse()
        .map_err(|error| format!("{CONFIG} isn't valid toml: {error}"))?)
}

pub fn push(doc: &mut DocumentMut, key: &str, names: &[String]) -> Result {
    let array = doc
        .get_mut(key)
        .and_then(Item::as_array_mut)
        .ok_or_else(|| format!("{CONFIG} has no `{key}` list"))?;
    for name in unique(names) {
        if !array.iter().any(|value| value.as_str() == Some(name)) {
            array.push(name);
        }
    }
    Ok(())
}

pub fn pull(doc: &mut DocumentMut, key: &str, names: &[String]) -> Result {
    let array = doc
        .get_mut(key)
        .and_then(Item::as_array_mut)
        .ok_or_else(|| format!("{CONFIG} has no `{key}` list"))?;
    let names = unique(names);
    array.retain(|value| !value.as_str().is_some_and(|value| names.contains(&value)));
    Ok(())
}

pub fn unique(names: &[String]) -> Vec<&str> {
    let mut out: Vec<&str> = Vec::new();
    for name in names.iter().map(|name| name.trim()) {
        if !name.is_empty() && !out.contains(&name) {
            out.push(name);
        }
    }
    out
}
