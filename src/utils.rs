use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use procfs::process::Process;
use chrono;
use dirs::home_dir;
use std::env;
use std::sync::OnceLock;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::Write;

static IMAGES_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn set_images_dir(images_dir: PathBuf) {
    let path = images_dir.as_path();
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
    IMAGES_DIR.set(PathBuf::from(expanded_path)).unwrap();
}

pub fn get_images_dir() -> PathBuf {
    IMAGES_DIR.get().unwrap().clone()
}       

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckpointMeta {
    pub checkpoint_id: String,
    pub pid: i32,
    pub cmd: String,
    pub dump_time: String,
}

impl CheckpointMeta {
    pub fn new(pid: i32) -> Self {
        let cmd = get_process_cmd(pid);
        let dump_time = chrono::Utc::now().to_string();

        let mut meta = CheckpointMeta {
            checkpoint_id: String::new(),
            pid,
            cmd,
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