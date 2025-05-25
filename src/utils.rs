use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fs::File;
use std::path::Path;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckpointMeta {
    pub checkpoint_id: String,
    pub pid: i32,
    pub cmd: String,
    pub dump_time: String,
}

impl CheckpointMeta {
    pub fn new(pid: i32, cmd: String, dump_time: String) -> Self {
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
