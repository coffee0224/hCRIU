use crate::utils;
use rust_criu::Criu;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

pub fn handle_restore(criu: &mut Criu, checkpoint_id: String) {
    let checkpoint_dir = find_by_prefix(&checkpoint_id);
    let image_dir = checkpoint_dir.join("image");
    let image_fd = std::fs::File::open(&image_dir).unwrap();
    criu.set_images_dir_fd(image_fd.as_raw_fd());
    criu.set_work_dir_fd(image_fd.as_raw_fd());
    criu.set_log_level(4);
    criu.set_log_file("restore.log".to_string());
    criu.set_shell_job(true);

    criu.restore()
        .map_err(|e| {
            eprintln!("Failed to restore: {}", e);
            std::process::exit(1);
        })
        .and_then(|_| {
            println!("Restore Success");
            Ok(())
        })
        .unwrap();
}

fn find_by_prefix(prefix: &str) -> PathBuf {
    if prefix.len() < 4 {
        eprintln!("Prefix must be at least 4 characters long");
        std::process::exit(1);
    }

    let hcriu_dir = utils::get_hcriu_dir();
    let mut checkpoints = Vec::new();
    for entry in std::fs::read_dir(hcriu_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir()
            && path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(prefix)
        {
            checkpoints.push(path);
        }
    }

    if checkpoints.len() == 1 {
        checkpoints[0].clone()
    } else {
        eprintln!(
            "Ambiguous prefix: {} checkpoints match '{}':",
            checkpoints.len(),
            prefix
        );
        for checkpoint in checkpoints {
            println!("  {}", checkpoint.display());
        }
        std::process::exit(1);
    }
}
