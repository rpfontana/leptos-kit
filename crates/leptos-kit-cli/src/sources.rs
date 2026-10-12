use crate::config::{CONFIG, Config};
use crate::registry::{COMPONENTS, Component, ICONS, UTILS};
use crate::{Result, slashes, write_new};
use std::fs;

pub fn write(config: &Config, order: &[&Component], overwrite: &[String]) -> Result {
    let dir = &config.components_dir;
    fs::create_dir_all(dir)?;
    write_new(&dir.join("utils.rs"), UTILS)?;
    let modules_path = dir.join("mod.rs");
    let mut modules = fs::read_to_string(&modules_path).unwrap_or_else(|_| "mod utils;\n".into());
    let icons = icons(&config.icons);
    for component in order {
        let file = dir.join(format!("{}.rs", component.module()));
        if !file.exists() || overwrite.iter().any(|name| name == component.name) {
            fs::write(
                &file,
                resolve_icons(component.source, &config.icons, &icons),
            )?;
            println!("wrote {}", slashes(&file));
        }
        let module = format!("pub mod {};", component.module());
        if !modules.lines().any(|line| line.trim() == module) {
            modules.push_str(&module);
            modules.push('\n');
        }
    }
    for component in COMPONENTS {
        let file = dir.join(format!("{}.rs", component.module()));
        let Ok(source) = fs::read_to_string(&file) else {
            continue;
        };
        if order.iter().any(|kept| kept.name == component.name) {
            continue;
        }
        if source != resolve_icons(component.source, &config.icons, &icons) {
            println!(
                "kept {}: it has your changes, but {CONFIG} doesn't list it, so it has no css",
                slashes(&file)
            );
            continue;
        }
        fs::remove_file(&file)?;
        let module = format!("pub mod {};", component.module());
        modules = modules
            .lines()
            .filter(|line| line.trim() != module)
            .map(|line| format!("{line}\n"))
            .collect();
        println!("removed {}", slashes(&file));
    }
    fs::write(&modules_path, modules)?;
    Ok(())
}

fn icons(library: &str) -> Vec<(String, String)> {
    let prefix = format!("pub use crate::{library}::");
    let mut icons: Vec<(String, String)> = ICONS
        .lines()
        .filter_map(|line| line.trim().strip_prefix(prefix.as_str()))
        .filter_map(|rest| rest.trim_end_matches(';').split_once(" as "))
        .map(|(glyph, name)| (glyph.to_string(), name.to_string()))
        .collect();
    icons.sort_by_key(|(_, name)| std::cmp::Reverse(name.len()));
    icons
}

fn resolve_icons(source: &str, library: &str, icons: &[(String, String)]) -> String {
    let mut out = source.replace(
        "use leptos_kit_icons::{Icon, semantic};",
        &format!("use leptos_kit_icons::{{Icon, {library}}};"),
    );
    for (glyph, name) in icons {
        out = out.replace(&format!("semantic::{name}"), &format!("{library}::{glyph}"));
    }
    out
}
