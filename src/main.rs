use clap::{Parser, Subcommand, ValueEnum};
use humantime::Duration;
use rust_criu::Criu;
use which::which;

mod dump;
mod list;
mod merge;
mod restore;
mod tui;
mod utils;

#[derive(Debug, ValueEnum, Clone)]
enum Sort {
    Time,
    Pid,
}

#[derive(Debug, Parser)]
#[command(name = "hCRIU")]
#[command(about = "checkpoint management tool")]
struct Cli {
    /// Specify CRIU executable path
    #[arg(long)]
    criu_path: Option<String>,

    /// Specify checkpoints directory, where store all checkpoints
    #[arg(short = 'D', long, default_value = "~/.hcriu/")]
    hcriu_dir: String,

    /// Use TUI output for list/merge
    #[arg(short = 't', long, default_value = "false")]
    tui: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create a new checkpoint
    Dump {
        /// checkpoint process tree identifier by  PID
        pid: i32,

        /// Create interval (e.g., 10s, 30m, 1h)
        #[arg(short, long)]
        interval: Option<Duration>,

        /// Create checkpoint with a tag
        #[arg(short, long)]
        tag: Option<String>,

        /// leave running processes before creation
        #[arg(long, default_value = "false")]
        leave_running: bool,
    },

    /// Restore container from checkpoint
    Restore { checkpoint_id: String },

    /// List all checkpoints
    List {
        /// Sort checkpoints by time or pid
        #[arg(long, default_value = "time")]
        sort: Sort,
    },

    /// Merge checkpoints, by default, it will keep the latest checkpoint
    Merge {
        /// tag filter for checkpoints to merge
        tag: String,

        /// do not merge, just print the result
        #[arg(short, long, default_value = "false")]
        dry_run: bool,

        /// pid filter for the process to merge
        #[arg(short, long)]
        pid: Option<i32>,

        /// keep daily checkpoints
        #[arg(long, default_value = "false")]
        keep_daily: bool,

        /// keep hourly checkpoints
        #[arg(long, default_value = "false")]
        keep_hourly: bool,
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

    utils::set_hcriu_dir(cli.hcriu_dir.into());
    let hcriu_dir = utils::get_hcriu_dir();
    if !hcriu_dir.exists() {
        std::fs::create_dir_all(hcriu_dir).unwrap();
    }

    match cli.command {
        Commands::Dump {
            pid,
            interval,
            tag,
            leave_running,
        } => {
            dump::handle_dump(&mut criu, pid, interval, tag, leave_running);
        }
        Commands::Restore { checkpoint_id } => {
            restore::handle_restore(&mut criu, checkpoint_id);
        }
        Commands::List { sort } => {
            list::handle_list(sort, cli.tui);
        }
        Commands::Merge {
            tag,
            dry_run,
            pid,
            keep_daily,
            keep_hourly,
        } => {
            merge::handle_merge(tag, dry_run, pid, keep_daily, keep_hourly, cli.tui);
        }
    }
}
