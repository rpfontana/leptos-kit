//! maintenance tasks, run with `cargo xtask <task>`

mod icons;

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

const TASKS: &[(&str, &str, fn(&[String]) -> Result)] = &[(
    "icons",
    "regenerate the icon data in leptos-kit-icons",
    icons::run,
)];

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

fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the workspace root")
        .to_path_buf()
}
