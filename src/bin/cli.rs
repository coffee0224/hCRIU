use clap::{Parser, Subcommand, CommandFactory};
use humantime::Duration;
use rust_criu::Criu;
use std::error::Error;
use which::which;
use hcriu::{dump, list, merge, restore, utils, Sort};


#[derive(Debug, Parser)]
#[command(name = "hcriu")]
#[command(about = "hCRIU is a checkpoint management tool based CRIU")]
struct Cli {
  /// Specify CRIU executable path
  #[arg(long)]
  path: Option<String>,

  /// Specify checkpoints directory, where store all checkpoints
  #[arg(short = 'd', long, default_value = "~/.hcriu/")]
  dir: String,

  #[command(subcommand)]
  command: Option<Commands>,
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

fn handle_command(criu: &mut Criu, cli: &Cli) -> Result<(), Box<dyn Error>> {
  match &cli.command {
    Some(Commands::Dump {
      pid,
      interval,
      tag,
      leave_running,
    }) => {
      dump::handle_dump(criu, *pid, interval.clone(), tag.clone(), *leave_running);
      Ok(())
    }
    Some(Commands::Restore { checkpoint_id }) => {
      restore::handle_restore(criu, checkpoint_id.clone());
      Ok(())
    }
    Some(Commands::List { sort }) => {
      list::handle_list(sort.to_owned());
      Ok(())
    }
    Some(Commands::Merge {
      tag,
      dry_run,
      pid,
      keep_daily,
      keep_hourly,
    }) => {
      merge::handle_merge(
        tag.clone(),
        *dry_run,
        *pid,
        *keep_daily,
        *keep_hourly,
      );
      Ok(())
    }
    None => {
      Cli::command().print_help().unwrap();
      Ok(())
    }
  }
}

fn main() {
  let cli = Cli::parse();

  // Find CRIU path if not provided
  let path = match &cli.path {
    Some(path) => path.clone(),
    None => match find_criu_path() {
      Some(path) => path,
      None => {
        eprintln!("criu not found in PATH, please specify --criu-path");
        std::process::exit(1);
      }
    },
  };

  let mut criu = Criu::new_with_criu_path(path).unwrap();
  utils::set_hcriu_dir(cli.dir.clone().into());
  let dir = utils::get_hcriu_dir();
  if !dir.exists() {
    std::fs::create_dir_all(dir).unwrap();
  }

  handle_command(&mut criu, &cli).unwrap();
}
