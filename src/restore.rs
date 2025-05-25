use rust_criu::Criu;

pub fn handle_restore(criu: &mut Criu, checkpoint_id: String) {
    criu.set_shell_job(true);
    criu.restore().map_err(|e| {
        eprintln!("Failed to restore: {}", e);
        std::process::exit(1);
    }).and_then(|_| {
        println!("Restore Success");
        Ok(())
    }).unwrap();
}