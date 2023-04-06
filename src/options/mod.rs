pub mod policy_choice;
pub mod operation;
mod log_level_filter;
mod card_probabilities;
mod hand_info;

pub use log_level_filter::*;
pub use card_probabilities::*;

use clap::{Args, Parser, Subcommand};
use log::LevelFilter;
use crate::options::operation::Operation;

#[derive(Parser)]
pub struct Cli{
    #[command(subcommand, rename_all = "snake_case")]
    pub command: Operation,
    #[arg(short = 'l', long = "log", default_value_t= LevelFilter::Info)]
    pub log_level: LevelFilter

}
