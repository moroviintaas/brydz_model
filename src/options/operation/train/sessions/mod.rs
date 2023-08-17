mod simple;

use std::path::PathBuf;
pub use simple::*;
use crate::options::operation::TrainOptions;
use clap::Subcommand;


#[derive(Subcommand)]
pub enum TrainType{
    Simple(TrainOptions)
}