use clap::{Parser, Subcommand, ValueEnum};
use rust_criu::Criu;
use which::which;

mod dump;
mod restore;
mod utils;

#[derive(Debug, ValueEnum, Clone)]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Parser)]
#[command(name = "hCRIU")]
#[command(about = "checkpoint management tool")]
struct Cli {
    /// Specify configuration file path
    // #[arg(short, long, default_value = "~/.checkpointctl.yaml")]
    // config: String,

    /// Specify CRIU executable path
    #[arg(long)]
    criu_path: Option<String>,

    /// Specify images directory, where store all checkpoints
    #[arg(short = 'D', long, default_value = "~/.hcriu/")]
    images_dir: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create a new checkpoint
    Dump {
        /// checkpoint process tree identifier by  PID
        pid: i32,

        /// Create interval (e.g., 30m, 1h)
        // #[arg(short, long)]
        // interval: Option<Duration>,

        /// Add metadata label (can be used multiple times)
        // #[arg(short, long)]
        // label: Vec<String>,

        /// leave running processes before creation
        #[arg(long, default_value = "false")]
        leave_running: bool,
    },

    /// Restore container from checkpoint
    Restore {
        checkpoint_id: String,
    },

    List {
    },
}

fn find_criu_path() -> Option<String> {
    which("criu").ok().map(|p| p.to_string_lossy().into_owned())
}

fn main() {
    let cli = Cli::parse();
    
    // Find CRIU path if not provided
    let criu_path = if cli.criu_path.is_none() {
        if let Some(path) = find_criu_path() {
            path
        } else {
            eprintln!("criu not found in PATH, please specify --criu-path");
            std::process::exit(1);
        }
    } else {
        cli.criu_path.clone().unwrap()
    };
    
    let mut criu = Criu::new_with_criu_path(criu_path).unwrap();
    let version = criu.get_criu_version().unwrap();
    println!("CRIU version: {}", version);

    utils::set_images_dir(cli.images_dir.into());
    let images_dir = utils::get_images_dir();
    if !images_dir.exists() {
        std::fs::create_dir_all(images_dir).unwrap();
    }

    match cli.command {
        Commands::Dump { pid, leave_running } => {
            dump::handle_create(&mut criu, pid, leave_running);
        }
        Commands::Restore { checkpoint_id } => {
            restore::handle_restore(&mut criu, checkpoint_id);
        }
        _ => {}
    }
}
