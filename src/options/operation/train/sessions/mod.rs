mod simple;
mod contract_session;
mod dynamic;

pub use contract_session::*;
pub use simple::*;
pub use dynamic::*;
use crate::options::operation::TrainOptions;
use clap::Subcommand;


#[derive(Subcommand)]
pub enum TrainType{
    Simple(TrainOptions)
}