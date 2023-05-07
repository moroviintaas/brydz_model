pub mod policy_choice;
pub mod operation;
mod log_level_filter;
mod hand_info;

use std::path::PathBuf;
pub use log_level_filter::*;

use clap::{Parser};
use log::LevelFilter;
use crate::options::operation::Operation;

#[derive(Parser)]
pub struct Cli{
    #[command(subcommand, rename_all = "snake_case")]
    pub command: Operation,
    #[arg(short = 'l', long = "log", default_value_t= LevelFilter::Info)]
    pub log_level: LevelFilter,
    #[arg(long = "log_file")]
    pub log_file: Option<PathBuf>,

}
