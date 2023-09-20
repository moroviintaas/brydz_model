mod ac_generic;
mod team;
mod q_generic;
mod options;
mod main_session;
mod traits;

pub use ac_generic::*;
pub use team::*;
pub use q_generic::*;
pub use options::*;
pub use main_session::*;
pub use traits::*;


use crate::options::operation::TrainOptions;
use clap::Subcommand;
use brydz_core::sztorm::spec::ContractDP;
use brydz_core::sztorm::state::{ContractAgentInfoSetSimple, ContractInfoSetConvert420Normalised};
use sztorm::error::SztormError;
use sztorm_rl::error::SztormRLError;


#[derive(Subcommand)]
pub enum TrainType{
    Simple(TrainOptions)
}


pub fn build_and_run_train_session(agent_type: &AgentType) -> Result<(), SztormRLError<ContractDP>>{
    match agent_type{
        AgentType::ActorCritic(options) => {
            let mut session = t_session_a2c_symmetric::<ContractAgentInfoSetSimple, ContractInfoSetConvert420Normalised>(options)?;
            session.train_all_at_once(options.epochs as usize, options.games as usize, options.tests_set_size as usize, None, &Default::default())
            //train_session_a2c(options)
        }
        AgentType::Q(options) => {
            let mut session = t_session_q_symmetric::<ContractAgentInfoSetSimple, ContractInfoSetConvert420Normalised>(options)?;
            session.train_all_at_once(options.epochs as usize, options.games as usize, options.tests_set_size as usize, None, &Default::default())
            //train_session_q(options)
        }
    }
}