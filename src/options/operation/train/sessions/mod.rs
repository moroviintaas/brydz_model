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
use brydz_core::sztorm::spec::ContractDP;
use sztorm::error::SztormError;


#[derive(Subcommand)]
pub enum TrainType{
    Simple(TrainOptions)
}


pub fn build_and_run_train_session(agent_type: &AgentType) -> Result<(), SztormError<ContractDP>>{
    match agent_type{
        AgentType::ActorCritic(options) => {
            train_session_a2c(options)
        }
        AgentType::Q(options) => {
            train_session_q(options)
        }
    }
}