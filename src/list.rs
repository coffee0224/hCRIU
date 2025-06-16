use crate::{Sort, utils};

pub fn handle_list(sort: Sort) {
  let mut checkpoints = utils::get_all_checkpoints();
  match sort {
    Sort::Time => checkpoints.sort_by(|a, b| a.dump_time.cmp(&b.dump_time)),
    Sort::Pid => checkpoints.sort_by(|a, b| a.pid.cmp(&b.pid)),
  }
  utils::print_checkpoints_table(checkpoints.iter().collect());
}
