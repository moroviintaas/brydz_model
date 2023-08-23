mod simple;
mod contract_session;

pub use contract_session::*;
pub use simple::*;
use crate::options::operation::TrainOptions;
use clap::Subcommand;


#[derive(Subcommand)]
pub enum TrainType{
    Simple(TrainOptions)
}