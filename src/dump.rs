use rust_criu::Criu;
use crate::utils;
use std::os::unix::io::AsRawFd;

pub fn handle_create(criu: &mut Criu, pid: i32, leave_running: bool) {
    let meta = utils::CheckpointMeta::new(pid);
    let checkpoint_dir = utils::get_hcriu_dir().join(meta.checkpoint_id.clone());
    if !checkpoint_dir.exists() {
        std::fs::create_dir_all(&checkpoint_dir).unwrap();
    } else {
        eprintln!("Checkpoint {} already exists", meta.checkpoint_id);
        std::process::exit(1);
    }

    let meta_file = checkpoint_dir.join("meta.toml");
    meta.save(&meta_file).unwrap();

    let image_dir = checkpoint_dir.join("image");
    std::fs::create_dir_all(&image_dir).unwrap();
    let image_fd = std::fs::File::open(&image_dir).unwrap();
    criu.set_images_dir_fd(image_fd.as_raw_fd());
    criu.set_work_dir_fd(image_fd.as_raw_fd());

    criu.set_log_level(4);
    criu.set_log_file("dump.log".to_string());

    criu.set_pid(pid);
    criu.set_leave_running(leave_running);
    criu.set_shell_job(true);

    criu.dump().map_err(|e| {
        eprintln!("Failed to dump: {}", e);
        std::process::exit(1);
    }).and_then(|_| {
        println!("Dump success to {}", checkpoint_dir.display());
        Ok(())
    }).unwrap();
}
