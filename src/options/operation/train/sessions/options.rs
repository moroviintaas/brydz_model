use clap::Subcommand;
use crate::options::operation::TrainOptions;

#[derive(Subcommand)]
pub enum AgentType{
    A2C(TrainOptions),
    Q(TrainOptions)
}

