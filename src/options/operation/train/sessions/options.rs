use clap::Subcommand;
use crate::options::operation::TrainOptions;

#[derive(Subcommand)]
pub enum AgentType{
    ActorCritic(TrainOptions),
    Q(TrainOptions)
}