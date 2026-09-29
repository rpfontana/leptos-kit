//! maintenance tasks, run with `cargo xtask <task>`

mod html;
mod icons;

use std::path::{Path, PathBuf};

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

const TASKS: &[(&str, &str, fn(&[String]) -> Result)] = &[
    (
        "html",
        "regenerate the HTML attribute table in leptos-kit-macros",
        html::run,
    ),
    (
        "icons",
        "regenerate the icon data in leptos-kit-icons",
        icons::run,
    ),
];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(task) = args.first() else {
        usage();
        return;
    };

    match TASKS.iter().find(|(name, ..)| name == task) {
        Some((_, _, run)) => {
            if let Err(error) = run(&args[1..]) {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
        None => {
            eprintln!("unknown task: {task}");
            usage();
            std::process::exit(1);
        }
    }
}

fn usage() {
    eprintln!("usage: cargo xtask <task>");
    for (name, description, _) in TASKS {
        eprintln!("  {name:<10} {description}");
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the workspace root")
        .to_path_buf()
}

/// downloads once into `target/xtask`, so `file_name` must change with the version
fn download(url: &str, file_name: &str) -> Result<PathBuf> {
    let cached = workspace_root()
        .join("target")
        .join("xtask")
        .join(file_name);
    if !cached.exists() {
        std::fs::create_dir_all(cached.parent().unwrap())?;
        println!("downloading {url}");
        let status = std::process::Command::new("curl")
            .args(["-sfL", url, "-o"])
            .arg(&cached)
            .status()?;
        if !status.success() {
            let _ = std::fs::remove_file(&cached);
            return Err(format!("could not download {url}").into());
        }
    }
    Ok(cached)
}
