use crate::utils;

pub fn handle_merge(
    tag: String,
    dry_run: bool,
    pid: Option<i32>,
    keep_daily: bool,
    keep_hourly: bool,
    tui: bool,
) {
    let all_checkpoints = utils::get_all_checkpoints();
    let filtered_checkpoints = all_checkpoints
        .iter()
        .filter(|c| c.tag == tag)
        .collect::<Vec<_>>();

    let mut filtered_checkpoints = if let Some(pid) = pid {
        filtered_checkpoints
            .iter()
            .filter(|c| c.pid == pid)
            .map(|c| *c)
            .collect::<Vec<_>>()
    } else {
        filtered_checkpoints
    };

    filtered_checkpoints.sort_by(|a, b| a.dump_time.cmp(&b.dump_time));

    // filter checkpoints by time
    let keep_checkpoints = if keep_daily {
        // keep the latest checkpoint of each day
        let mut daily_checkpoints = Vec::new();
        let mut current_day = String::new();
        for checkpoint in filtered_checkpoints.iter().rev() {
            let day = checkpoint.dump_time.split(' ').next().unwrap();
            if day != current_day {
                daily_checkpoints.push(*checkpoint);
                current_day = day.to_string();
            }
        }
        daily_checkpoints
    } else if keep_hourly {
        // keep the latest checkpoint of each hour
        let mut hourly_checkpoints = Vec::new();
        let mut current_hour = String::new();
        for checkpoint in filtered_checkpoints.iter().rev() {
            let hour = checkpoint
                .dump_time
                .split(' ')
                .nth(1)
                .unwrap()
                .split(':')
                .next()
                .unwrap();
            if hour != current_hour {
                hourly_checkpoints.push(*checkpoint);
                current_hour = hour.to_string();
            }
        }
        hourly_checkpoints
    } else {
        // keep only the latest checkpoint
        vec![
            *filtered_checkpoints
                .iter()
                .max_by_key(|c| &c.dump_time)
                .unwrap(),
        ]
    };

    if keep_checkpoints.is_empty() {
        eprintln!("No checkpoints to merge");
        std::process::exit(1);
    }

    // Fix: Use Vec<&CheckpointMeta> since .iter() returns references
    let merged_checkpoints = all_checkpoints
        .iter()
        .filter(|c| !keep_checkpoints.contains(c))
        .collect::<Vec<_>>();

    if dry_run {
        if tui {
            use crate::tui::show_checkpoints_tui;
            println!("The following checkpoints will be merged:");
            show_checkpoints_tui(merged_checkpoints);
            println!("The following checkpoints will be kept:");
            show_checkpoints_tui(keep_checkpoints);
        } else {
            println!("The following checkpoints will be merged:");
            utils::print_checkpoints_table(merged_checkpoints);
            println!("The following checkpoints will be kept:");
            utils::print_checkpoints_table(keep_checkpoints);
        }
    } else {
        merged_checkpoints.iter().for_each(|c| {
            // delete the checkpoint
            let checkpoint_dir = utils::get_hcriu_dir().join(c.checkpoint_id.clone());
            std::fs::remove_dir_all(&checkpoint_dir).unwrap();
            println!("Deleted checkpoint {}", c.checkpoint_id);
        });
        println!("Merged {:?} checkpoints", merged_checkpoints.len());
    }
}