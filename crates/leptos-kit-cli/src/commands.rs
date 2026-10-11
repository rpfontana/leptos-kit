use crate::config::{self, CONFIG, Config};
use crate::registry::COMPONENTS;
use crate::{Change, Init, Result, css, slashes, sources};
use std::fs;
use std::path::Path;
use std::process::Command;
use toml_edit::DocumentMut;

const CRATES: &[&str] = &[
    "leptos-kit-primitives",
    "leptos-kit-macros",
    "leptos-kit-icons",
];

pub fn init(args: Init) -> Result {
    let quoted = |names: &[String]| {
        config::unique(names)
            .iter()
            .map(|name| format!("\"{name}\""))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let text = format!(
        "styles = [{}]\nbase-colors = [{}]\nicons = \"{}\"\ncomponents = []\n\n[paths]\ncomponents = \"{}\"\ncss = \"{}\"\n",
        quoted(&args.style),
        quoted(&args.base_color),
        args.icons,
        slashes(&args.components),
        slashes(&args.css),
    );
    let doc: DocumentMut = text.parse()?;
    let config = Config::from(&doc)?;
    if Path::new(CONFIG).exists() {
        return Err(format!("{CONFIG} already exists").into());
    }

    match &args.kit {
        Some(kit) => {
            for name in CRATES {
                let path = kit.join("crates").join(name);
                let mut cargo = Command::new("cargo");
                cargo.arg("add").arg("--path").arg(&path);
                if *name == "leptos-kit-icons" {
                    cargo.args(["--no-default-features", "--features", &config.icons]);
                }
                if !cargo.status()?.success() {
                    return Err(format!("`cargo add --path {}` failed", slashes(&path)).into());
                }
            }
        }
        None => println!(
            "add the dependencies: {}, with the `{}` feature of leptos-kit-icons",
            CRATES.join(", "),
            config.icons
        ),
    }
    write(doc, &[])?;

    let module = config
        .components_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("ui");
    println!(
        "\nleptos-kit is set up, three things left to you:\n\
         - `mod {module};` in the crate root\n\
         - `@import \"…/{}/index.css\";` in the Tailwind entry, after tailwindcss\n\
         - `leptos-kit-primitives/ssr` and `leptos-kit-icons/ssr` in the ssr feature, `/hydrate` in hydrate",
        slashes(&config.css_dir),
    );
    Ok(())
}

pub fn add(change: Change, overwrite: bool) -> Result {
    if change.is_empty() {
        return Err("add what? components, `--style` or `--base-color`".into());
    }
    let mut doc = config::load()?;
    config::push(&mut doc, "components", &change.components)?;
    config::push(&mut doc, "styles", &change.style)?;
    config::push(&mut doc, "base-colors", &change.base_color)?;
    let overwrite = match overwrite {
        true => change.components,
        false => Vec::new(),
    };
    write(doc, &overwrite)
}

pub fn remove(change: Change) -> Result {
    if change.is_empty() {
        return Err("remove what? components, `--style` or `--base-color`".into());
    }
    let mut doc = config::load()?;
    config::pull(&mut doc, "components", &change.components)?;
    config::pull(&mut doc, "styles", &change.style)?;
    config::pull(&mut doc, "base-colors", &change.base_color)?;
    let config = Config::from(&doc)?;
    let order = config.order()?;
    if let Some(needed) = order
        .iter()
        .find(|component| change.components.iter().any(|name| name == component.name))
    {
        let user = order
            .iter()
            .find(|component| component.dependencies.contains(&needed.name))
            .map_or("another component", |component| component.name);
        return Err(format!("{} stays, {user} needs it", needed.name).into());
    }
    write(doc, &[])
}

pub fn apply(overwrite: bool) -> Result {
    let doc = config::load()?;
    let overwrite = match overwrite {
        true => Config::from(&doc)?
            .order()?
            .iter()
            .map(|component| component.name.to_string())
            .collect(),
        false => Vec::new(),
    };
    write(doc, &overwrite)
}

pub fn list() {
    for component in COMPONENTS {
        match component.dependencies {
            [] => println!("{}", component.name),
            dependencies => println!("{} (with {})", component.name, dependencies.join(", ")),
        }
    }
}

fn write(mut doc: DocumentMut, overwrite: &[String]) -> Result {
    let config = Config::from(&doc)?;
    let order = config.order()?;
    let needed: Vec<String> = order
        .iter()
        .map(|component| component.name.to_string())
        .filter(|name| !config.components.contains(name))
        .collect();
    if !needed.is_empty() {
        config::push(&mut doc, "components", &needed)?;
        println!(
            "added {} to {CONFIG}, the components there need it",
            needed.join(", ")
        );
    }
    fs::write(CONFIG, doc.to_string())?;
    css::write(&config, &order)?;
    sources::write(&config, &order, overwrite)?;
    classes(&config);
    Ok(())
}

fn classes(config: &Config) {
    if config.styles.len() > 1 {
        println!(
            "several styles: `class=\"style-{}\"` on <html>, or on any element, picks the one inside it",
            config.styles[0]
        );
    }
    if let [first, others @ ..] = config.base_colors.as_slice()
        && !others.is_empty()
    {
        println!(
            "{first} is the default colour, `class=\"base-color-{}\"` on <html> or any element switches what is inside it",
            others[0]
        );
    }
}
