use chrono;
use comfy_table::Table;
use dirs::home_dir;
use procfs::process::Process;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static HCRIU_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn set_hcriu_dir(hcriu_dir: PathBuf) {
  let path = hcriu_dir.as_path();
  let expanded_path = if path.starts_with("~") {
    if let Ok(sudo_user) = env::var("SUDO_USER") {
      #[cfg(unix)]
      let home = PathBuf::from(format!("/home/{}", sudo_user));
      home.join(path.strip_prefix("~").unwrap())
    } else {
      home_dir().unwrap().join(path.strip_prefix("~").unwrap())
    }
  } else {
    path.to_path_buf()
  };
  HCRIU_DIR.set(PathBuf::from(expanded_path)).unwrap();
}

pub fn get_hcriu_dir() -> PathBuf {
  HCRIU_DIR.get().unwrap().clone()
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct CheckpointMeta {
  pub checkpoint_id: String,
  pub pid: i32,
  pub cmd: String,
  pub tag: String,
  pub dump_time: String,
}

impl CheckpointMeta {
  pub fn new(pid: i32, tag: &Option<String>) -> Self {
    let cmd = get_process_cmd(pid);
    let dump_time = chrono::Utc::now().to_string();
    let tag = if let Some(tag) = tag {
      tag.clone()
    } else {
      format!("tmp-{}", pid)
    };

    let mut meta = CheckpointMeta {
      checkpoint_id: String::new(),
      pid,
      cmd,
      tag,
      dump_time,
    };

    meta.update_checkpoint_id();
    meta
  }

  fn update_checkpoint_id(&mut self) {
    // concat pid + cmd + dump_start_time
    let input = format!("{}{}{}", self.pid, self.cmd, self.dump_time);

    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();

    self.checkpoint_id = format!("{:x}", hash);
  }

  pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
    let toml = toml::to_string(self).unwrap();
    let mut file = File::create(path)?;
    file.write_all(toml.as_bytes())?;
    Ok(())
  }

  pub fn parse(meta: String) -> CheckpointMeta {
    let meta: CheckpointMeta = toml::from_str(&meta).unwrap();
    meta
  }
}

fn get_process_cmd(pid: i32) -> String {
  let process = Process::new(pid).unwrap();
  process.cmdline().unwrap().join(" ")
}

pub fn get_all_checkpoints() -> Vec<CheckpointMeta> {
  let hcriu_dir = get_hcriu_dir();
  std::fs::read_dir(hcriu_dir)
    .unwrap()
    .map(|c| {
      let checkpoint = c.unwrap();
      let meta_file = checkpoint.path().join("meta.toml");
      let meta = CheckpointMeta::parse(std::fs::read_to_string(meta_file).unwrap());
      meta
    })
    .collect()
}

pub fn print_checkpoints_table(checkpoints: Vec<&CheckpointMeta>) {
  let mut table = Table::new();
  table.set_header(vec!["Checkpoint ID", "Tag", "PID", "Command", "Dump Time"]);
  for checkpoint in checkpoints {
    table.add_row(vec![
      checkpoint.checkpoint_id[..7].to_string(),
      checkpoint.tag.clone(),
      checkpoint.pid.to_string(),
      checkpoint.cmd.clone(),
      checkpoint.dump_time.clone(),
    ]);
  }
  println!("{}", table);
}
