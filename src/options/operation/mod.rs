mod gen_contract;
pub mod test_ops;
pub mod simulate_local;
pub mod train;
pub mod demo_op;
pub mod generate;


pub use gen_contract::*;
use clap::Subcommand;
use crate::options::operation::demo_op::DemoCommands;
use crate::options::operation::simulate_local::SimContractOptions;
use crate::options::operation::train::sessions::AgentType;

#[derive(Subcommand)]
pub enum Operation {

    ContractGen(GenContractOptions),
    LocalSimContract(SimContractOptions),
    //TestLocal,
    //TestTcp,
    //TestGeneric,
    //TestRunNN,
    //TestBiasedParams,
    #[command(subcommand, rename_all = "snake_case")]
    Train(AgentType),
    #[command(subcommand, rename_all = "snake_case")]
    Demo(DemoCommands),
}