mod simple;
mod contract_session;
mod dynamic;
mod generic;

pub use contract_session::*;
pub use simple::*;
pub use dynamic::*;
pub use generic::*;


use crate::options::operation::TrainOptions;
use clap::Subcommand;



#[derive(Subcommand)]
pub enum TrainType{
    Simple(TrainOptions)
}