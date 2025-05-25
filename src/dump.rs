use rust_criu::Criu;

pub fn handle_create(criu: &mut Criu, pid: i32, leave_running: bool) {
    // TODO: Implement create command logic
    println!("pid: {}", pid);
    println!("leave_running: {}", leave_running);


    criu.set_pid(pid);
    criu.set_leave_running(leave_running);
    criu.set_shell_job(true);

    criu.dump().map_err(|e| {
        eprintln!("Failed to dump: {}", e);
        std::process::exit(1);
    }).and_then(|_| {
        println!("Dump Success");
        Ok(())
    }).unwrap();
}
