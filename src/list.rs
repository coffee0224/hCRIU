use crate::utils;
use crate::Sort;
use comfy_table::Table;

pub fn handle_list(sort: Sort) {
    let mut table = Table::new();
    table.set_header(vec!["Checkpoint ID", "PID", "Command", "Dump Time"]);
    
    let mut checkpoints = utils::get_all_checkpoints();

    match sort {
        Sort::Time => {
            checkpoints.sort_by(|a, b| a.dump_time.cmp(&b.dump_time));
        }
        Sort::Pid => {
            checkpoints.sort_by(|a, b| a.pid.cmp(&b.pid));
        }
    }
    for meta in checkpoints {
        table.add_row(vec![
            meta.checkpoint_id[..7].to_string(),
            meta.pid.to_string(),
            meta.cmd,
            meta.dump_time,
        ]);
    }
    println!("{}", table);
}