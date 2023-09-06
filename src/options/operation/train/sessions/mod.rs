mod generic;
mod team;
mod q_generic;
mod options;

pub use generic::*;
pub use team::*;
pub use q_generic::*;
pub use options::*;


use crate::options::operation::TrainOptions;
use clap::Subcommand;



#[derive(Subcommand)]
pub enum TrainType{
    Simple(TrainOptions)
}