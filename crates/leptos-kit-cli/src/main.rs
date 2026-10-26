mod commands;
mod config;
mod css;
mod files;
mod registry;
mod sources;

use clap::{Args, Parser, Subcommand};
use std::path::{Path, PathBuf};

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[derive(Parser)]
#[command(
    name = "leptos-kit",
    version,
    about = "Adds leptos-kit components to a Leptos app",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    task: Task,
}

#[derive(Subcommand)]
enum Task {
    #[command(about = "Write leptos-kit.toml and set the app up")]
    Init(Init),
    #[command(about = "Add components, styles or base colours to leptos-kit.toml, then apply")]
    Add {
        #[command(flatten)]
        change: Change,
        #[arg(long, help = "Copy the named components' sources again")]
        overwrite: bool,
    },
    #[command(about = "Remove components, styles or base colours from leptos-kit.toml, then apply")]
    Remove(Change),
    #[command(about = "Make the app's files match leptos-kit.toml")]
    Apply {
        #[arg(long, help = "Copy every component's source again")]
        overwrite: bool,
    },
    #[command(about = "List the components")]
    List,
}

#[derive(Args)]
struct Init {
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "nova",
        help = "One style or several, as nova,lyra"
    )]
    style: Vec<String>,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "neutral",
        help = "One base colour or several, the first is the default"
    )]
    base_color: Vec<String>,
    #[arg(long, default_value = "lucide")]
    icons: String,
    #[arg(
        long,
        default_value = "src/ui",
        help = "Where the components' sources go"
    )]
    components: PathBuf,
    #[arg(long, default_value = "style/leptos-kit", help = "Where the css goes")]
    css: PathBuf,
    #[arg(
        long,
        value_name = "PATH",
        help = "A leptos-kit checkout, to add its crates by path"
    )]
    kit: Option<PathBuf>,
}

#[derive(Args)]
struct Change {
    #[arg(help = "Components, as `leptos-kit list` names them")]
    components: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    style: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    base_color: Vec<String>,
}

impl Change {
    fn is_empty(&self) -> bool {
        self.components.is_empty() && self.style.is_empty() && self.base_color.is_empty()
    }
}

fn main() {
    let result = match Cli::parse().task {
        Task::Init(args) => commands::init(args),
        Task::Add { change, overwrite } => commands::add(change, overwrite),
        Task::Remove(change) => commands::remove(change),
        Task::Apply { overwrite } => commands::apply(overwrite),
        Task::List => {
            commands::list();
            Ok(())
        }
    };
    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn slashes(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}
