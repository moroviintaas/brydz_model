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
pub struct CliOptions {
    #[command(subcommand, rename_all = "snake_case")]
    pub command: Operation,
    #[arg(short = 'l', long = "log", default_value_t= LevelFilter::Info)]
    pub log_level: LevelFilter,
    #[arg(short = 'c', long = "log_core", default_value_t= LevelFilter::Off)]
    pub brydz_core_log_level: LevelFilter,
    #[arg(short = 's', long = "log_sztorm", default_value_t= LevelFilter::Off)]
    pub sztorm_log_level: LevelFilter,
    #[arg(short = 'r', long = "log_sztorm-rl", default_value_t= LevelFilter::Off)]
    pub sztormrl_log_level: LevelFilter,

    #[arg(long = "log_file")]
    pub log_file: Option<PathBuf>,

}
