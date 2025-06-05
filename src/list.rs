use crate::{Sort, tui, utils};

pub fn handle_list(sort: Sort, tui: bool) {
  let mut checkpoints = utils::get_all_checkpoints();
  match sort {
    Sort::Time => checkpoints.sort_by(|a, b| a.dump_time.cmp(&b.dump_time)),
    Sort::Pid => checkpoints.sort_by(|a, b| a.pid.cmp(&b.pid)),
  }
  if tui {
    tui::show_checkpoints_tui(checkpoints.iter().collect());
  } else {
    utils::print_checkpoints_table(checkpoints.iter().collect());
  }
}
