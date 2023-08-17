mod gen_contract;
pub mod test_ops;
mod simulate_local;
mod train;
pub mod demo_op;
mod gen_distribution;


pub use gen_distribution::*;
pub use gen_contract::*;
use clap::Subcommand;
pub use simulate_local::*;
pub use train::*;
use crate::options::operation::demo_op::DemoCommands;
use crate::options::operation::sessions::TrainType;
//pub use demo_op::*;

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
    Train(TrainType),
    #[command(subcommand, rename_all = "snake_case")]
    Demo(DemoCommands),
}