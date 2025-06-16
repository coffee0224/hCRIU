pub mod dump;
pub mod list;
pub mod merge;
pub mod restore;
pub mod utils;

use clap::ValueEnum;

#[derive(Debug, ValueEnum, Clone)]
pub enum Sort {
  Time,
  Pid,
}