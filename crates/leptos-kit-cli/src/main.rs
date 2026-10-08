mod registry;

use registry::{
    BASE_COLORS, COMPONENTS, Component, ICON_LIBRARIES, ICONS, SHARED, STRUCTURE, STYLES, UTILS,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

const CONFIG: &str = "leptos-kit.toml";
// components take `aria-*` and `data-*` through this fork until leptos merges it
const PATCH: &str = r#"leptos_macro = { git = "https://github.com/rpfontana/leptos", branch = "dashed-attrs-on-components" }"#;
const CRATES: &[&str] = &[
    "leptos-kit-primitives",
    "leptos-kit-macros",
    "leptos-kit-icons",
];

const USAGE: &str = "usage:
  leptos-kit init [--style nova] [--base-color neutral] [--icons lucide]
                  [--components src/ui] [--css style/leptos-kit] [--kit <path to the leptos-kit repo>]
  leptos-kit add <component>... [--overwrite]
  leptos-kit list";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("init") => init(&args[1..]),
        Some("add") => add(&args[1..]),
        Some("list") => {
            list();
            Ok(())
        }
        None | Some("help" | "--help") => {
            println!("{USAGE}");
            Ok(())
        }
        Some(other) => Err(format!("unknown command `{other}`\n\n{USAGE}").into()),
    };
    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

struct Config {
    style: String,
    base_color: String,
    icons: String,
    components: PathBuf,
    css: PathBuf,
}

impl Config {
    fn read() -> Result<Self> {
        let text = fs::read_to_string(CONFIG)
            .map_err(|_| format!("no {CONFIG} here, run `leptos-kit init` first"))?;
        let value = |key: &str| -> Result<String> {
            text.lines()
                .filter_map(|line| line.split_once('='))
                .find(|(name, _)| name.trim() == key)
                .map(|(_, value)| value.trim().trim_matches('"').to_string())
                .ok_or_else(|| format!("{CONFIG} has no `{key}`").into())
        };
        Ok(Self {
            style: value("style")?,
            base_color: value("base-color")?,
            icons: value("icons")?,
            components: value("components")?.into(),
            css: value("css")?.into(),
        })
    }

    fn write(&self) -> Result {
        fs::write(
            CONFIG,
            format!(
                "style = \"{}\"\nbase-color = \"{}\"\nicons = \"{}\"\ncomponents = \"{}\"\ncss = \"{}\"\n",
                self.style,
                self.base_color,
                self.icons,
                slashes(&self.components),
                slashes(&self.css),
            ),
        )?;
        Ok(())
    }
}

fn init(args: &[String]) -> Result {
    let config = Config {
        style: option(args, "--style").unwrap_or("nova").into(),
        base_color: option(args, "--base-color").unwrap_or("neutral").into(),
        icons: option(args, "--icons").unwrap_or("lucide").into(),
        components: option(args, "--components").unwrap_or("src/ui").into(),
        css: option(args, "--css").unwrap_or("style/leptos-kit").into(),
    };
    pick(STYLES, &config.style, "style")?;
    let colors = pick(BASE_COLORS, &config.base_color, "base colour")?;
    if !ICON_LIBRARIES.contains(&config.icons.as_str()) {
        return Err(format!(
            "unknown icon library `{}`, expected one of {ICON_LIBRARIES:?}",
            config.icons
        )
        .into());
    }
    if Path::new(CONFIG).exists() {
        return Err(format!("{CONFIG} already exists").into());
    }

    match option(args, "--kit") {
        Some(kit) => {
            for name in CRATES {
                let path = Path::new(kit).join("crates").join(name);
                let status = Command::new("cargo")
                    .arg("add")
                    .arg("--path")
                    .arg(&path)
                    .status()?;
                if !status.success() {
                    return Err(format!("`cargo add --path {}` failed", slashes(&path)).into());
                }
            }
        }
        None => println!("add the dependencies: {}", CRATES.join(", ")),
    }
    patch_cargo()?;

    fs::create_dir_all(&config.css)?;
    for (name, css) in SHARED {
        fs::write(config.css.join(name), css)?;
    }
    fs::write(config.css.join("colors.css"), colors)?;
    write_new(&config.css.join("ui.css"), "")?;
    fs::write(
        config.css.join("index.css"),
        "@import \"./tw-animate.css\";\n@import \"./shadcn-tailwind.css\";\n@import \"./theme.css\";\n\
         @import \"./colors.css\";\n@import \"./leptos-kit.css\";\n@import \"./ui.css\";\n",
    )?;

    fs::create_dir_all(&config.components)?;
    write_new(&config.components.join("utils.rs"), UTILS)?;
    write_new(&config.components.join("mod.rs"), "mod utils;\n")?;
    config.write()?;

    let module = config
        .components
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("ui");
    println!(
        "\nleptos-kit is set up, three things left to you:\n\
         - `mod {module};` in the crate root\n\
         - `@import \"…/{}/index.css\";` in the Tailwind entry, after tailwindcss\n\
         - `leptos-kit-primitives/ssr` and `leptos-kit-icons/ssr` in the ssr feature, `/hydrate` in hydrate",
        slashes(&config.css),
    );
    Ok(())
}

fn add(args: &[String]) -> Result {
    let config = Config::read()?;
    let overwrite = args.iter().any(|arg| arg == "--overwrite");
    let names: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|arg| !arg.starts_with("--"))
        .collect();
    if names.is_empty() {
        return Err("which components? `leptos-kit list` shows them".into());
    }
    let mut order = Vec::new();
    for name in names {
        resolve(name, &mut order)?;
    }

    let style = pick(STYLES, &config.style, "style")?;
    let icons = icons(&config.icons);
    write_new(&config.components.join("utils.rs"), UTILS)?;
    let modules_path = config.components.join("mod.rs");
    let mut modules = fs::read_to_string(&modules_path).unwrap_or_else(|_| "mod utils;\n".into());
    let ui_path = config.css.join("ui.css");
    let mut ui = fs::read_to_string(&ui_path).unwrap_or_default();

    for component in order {
        let file = config.components.join(format!("{}.rs", component.name));
        if file.exists() && !overwrite {
            println!("kept {}", slashes(&file));
        } else {
            fs::write(
                &file,
                resolve_icons(component.source, &config.icons, &icons),
            )?;
            println!("wrote {}", slashes(&file));
        }
        let module = format!("pub mod {};", component.name);
        if !modules.lines().any(|line| line.trim() == module) {
            modules.push_str(&module);
            modules.push('\n');
        }
        match section(&ui, component.name) {
            Some(range) if overwrite => ui.replace_range(range, &css(component, style)),
            Some(_) => {}
            None => ui.push_str(&css(component, style)),
        }
    }
    fs::write(&modules_path, modules)?;
    fs::write(&ui_path, ui)?;
    Ok(())
}

fn list() {
    for component in COMPONENTS {
        match component.dependencies {
            [] => println!("{}", component.name),
            dependencies => println!("{} (with {})", component.name, dependencies.join(", ")),
        }
    }
}

// dependencies first, each once
fn resolve(name: &str, order: &mut Vec<&'static Component>) -> Result {
    let component = registry::component(name)
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

fn css(component: &Component, style: &str) -> String {
    let mut out = format!("/* {} */\n", component.name);
    let hooks = hooks(component.source);
    let base = rules(style, &hooks);
    if !base.is_empty() {
        out.push_str(&format!("@layer base {{\n{base}}}\n"));
    }
    let structure = rules(STRUCTURE, &hooks);
    if !structure.is_empty() {
        out.push_str(&format!("@layer components {{\n{structure}}}\n"));
    }
    out
}

fn section(css: &str, name: &str) -> Option<std::ops::Range<usize>> {
    let marker = format!("/* {name} */");
    let start = css
        .match_indices(&marker)
        .map(|(at, _)| at)
        .find(|&at| at == 0 || css[..at].ends_with('\n'))?;
    let end = css[start..]
        .find("\n/* ")
        .map_or(css.len(), |at| start + at + 1);
    Some(start..end)
}

// `cn-button-size-{size}` leaves the prefix `cn-button-size-`
fn hooks(source: &str) -> Vec<&str> {
    source
        .match_indices("cn-")
        .map(|(at, _)| {
            let rest = &source[at..];
            let end = rest
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '-')
                .unwrap_or(rest.len());
            &rest[..end]
        })
        .collect()
}

// the style sheets hold one rule per hook, two spaces in, under a single block
fn rules(css: &str, hooks: &[&str]) -> String {
    let mut out = String::new();
    let mut taking = false;
    for line in css.lines() {
        if line.starts_with("  .") && line.ends_with('{') {
            let hook: String = line[3..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect();
            taking = hooks.iter().any(|known| match known.strip_suffix('-') {
                Some(_) => hook.starts_with(known),
                None => hook == *known,
            });
        }
        if taking {
            out.push_str(line);
            out.push('\n');
            if line == "  }" {
                taking = false;
            }
        }
    }
    out
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

fn patch_cargo() -> Result {
    let text = fs::read_to_string("Cargo.toml")?;
    if text.contains("leptos_macro") {
        return Ok(());
    }
    let text = match text.find("[patch.crates-io]\n") {
        Some(at) => {
            let at = at + "[patch.crates-io]\n".len();
            format!("{}{PATCH}\n{}", &text[..at], &text[at..])
        }
        None => format!("{}\n[patch.crates-io]\n{PATCH}\n", text.trim_end()),
    };
    fs::write("Cargo.toml", text)?;
    println!("patched leptos_macro in Cargo.toml");
    Ok(())
}

fn pick(list: &'static [(&str, &str)], name: &str, what: &str) -> Result<&'static str> {
    list.iter()
        .find(|(known, _)| *known == name)
        .map(|(_, value)| *value)
        .ok_or_else(|| {
            let known: Vec<&str> = list.iter().map(|(known, _)| *known).collect();
            format!("unknown {what} `{name}`, expected one of {known:?}").into()
        })
}

fn option<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|at| args.get(at + 1))
        .map(String::as_str)
}

fn write_new(path: &Path, contents: &str) -> Result {
    if !path.exists() {
        fs::write(path, contents)?;
    }
    Ok(())
}

fn slashes(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}
