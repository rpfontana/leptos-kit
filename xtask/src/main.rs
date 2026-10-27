//! maintenance tasks, run with `cargo xtask <task>`
#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "a cli talks on stdout and stderr"
)]

mod html;
mod icons;
mod styles;

use std::path::{Path, PathBuf};

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

struct Task {
    name: &'static str,
    description: &'static str,
    run: fn(&[String]) -> Result,
}

const TASKS: &[Task] = &[
    Task {
        name: "html",
        description: "regenerate the HTML attribute table in leptos-kit-macros",
        run: html::run,
    },
    Task {
        name: "icons",
        description: "regenerate the icon data in leptos-kit-icons",
        run: icons::run,
    },
    Task {
        name: "styles",
        description: "vendor the shadcn style sheets into leptos-kit",
        run: styles::run,
    },
];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(task) = args.first() else {
        usage();
        return;
    };

    let Some(Task { run, .. }) = TASKS.iter().find(|known| known.name == task) else {
        eprintln!("unknown task: {task}");
        usage();
        std::process::exit(1);
    };
    if let Err(error) = run(&args[1..]) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn usage() {
    eprintln!("usage: cargo xtask <task>");
    for task in TASKS {
        eprintln!("  {:<10} {}", task.name, task.description);
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the workspace root")
        .to_path_buf()
}

// so a regenerated file passes `cargo fmt --check`
fn rustfmt(path: &Path) -> Result {
    let status = std::process::Command::new("rustfmt")
        .args(["--edition", "2024"])
        .arg(path)
        .status()?;
    if !status.success() {
        return Err(format!("rustfmt failed on {}", path.display()).into());
    }
    Ok(())
}

/// downloads once into `target/xtask`, so `file_name` must change with the version
fn download(url: &str, file_name: &str) -> Result<PathBuf> {
    let cache = workspace_root().join("target").join("xtask");
    let cached = cache.join(file_name);
    if !cached.exists() {
        std::fs::create_dir_all(&cache)?;
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
