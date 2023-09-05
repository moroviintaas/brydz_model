mod generic;
mod team;

pub use generic::*;
pub use team::*;


use crate::options::operation::TrainOptions;
use clap::Subcommand;



#[derive(Subcommand)]
pub enum TrainType{
    Simple(TrainOptions)
}