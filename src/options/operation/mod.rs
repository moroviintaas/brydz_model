mod gen_contract;
pub mod test_ops;
mod simulate_local;
mod train;


pub use gen_contract::*;
pub use clap::Subcommand;
pub use simulate_local::*;
pub use train::*;

#[derive(Subcommand)]
pub enum Operation {

    ContractGen(GenContractOptions),
    LocalSimContract(SimContractOptions),
    TestLocal,
    TestTcp,
    TestGeneric,
    TestRunNN,
    TestBiasedParams,
    Train(TrainOptions),
}