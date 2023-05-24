mod gen_contract;
pub mod test_ops;
mod simulate_local;
mod train;
pub mod test_op;


pub use gen_contract::*;
use clap::Subcommand;
pub use simulate_local::*;
pub use train::*;
use crate::options::operation::test_op::TestCommands;
//pub use test_op::*;

#[derive(Subcommand)]
pub enum Operation {

    ContractGen(GenContractOptions),
    LocalSimContract(SimContractOptions),
    //TestLocal,
    //TestTcp,
    //TestGeneric,
    //TestRunNN,
    //TestBiasedParams,
    Train(TrainOptions),
    #[command(subcommand, rename_all = "snake_case")]
    Test(TestCommands),
}