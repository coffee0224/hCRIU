use std::os::fd::AsRawFd;

use clap::{Parser, Subcommand, ValueEnum};
use humantime::Duration;
use bytesize::ByteSize;
use rust_criu::Criu;
use which::which;

mod dump;
mod restore;

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

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: LogLevel,

    #[arg(short = 'o', long)]
    log_file: Option<String>,

    /// Specify CRIU executable path
    #[arg(long)]
    criu_path: Option<String>,

    /// Specify image directory, used for dump and restore
    #[arg(short = 'D', long, default_value = "./")]
    image_dir: String,

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

    criu.set_log_level(match cli.log_level {
        LogLevel::Error => 0,
        LogLevel::Warn => 1,
        LogLevel::Info => 2,
        LogLevel::Debug => 3,
    });
    // if let Some(log_file) = cli.log_file {
    //     criu.set_log_file(log_file);
    // } else {
    //     let log_file = match cli.command {
    //         Commands::Dump { .. } => std::env::temp_dir().join("dump.log"),
    //         Commands::Restore { .. } => std::env::temp_dir().join("restore.log"),
    //         _ => unreachable!(),
    //     };
    //     criu.set_log_file(log_file.to_string_lossy().into_owned());
    // }

    let image_dir_fd = std::fs::File::open(&cli.image_dir).unwrap();
    criu.set_images_dir_fd(image_dir_fd.as_raw_fd());

    match cli.command {
        Commands::Dump { pid, leave_running } => {
            dump::handle_create(&mut criu, pid, leave_running);
        }
        Commands::Restore { } => {
            restore::handle_restore(&mut criu);
        }
        _ => {}
    }
}
