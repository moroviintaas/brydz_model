mod gen_contract;
pub mod test_ops;
mod simulate;


pub use gen_contract::*;
pub use clap::Subcommand;
pub use simulate::*;

#[derive(Subcommand)]
pub enum Operation {

    ContractGen(GenContractOptions),
    ContractSim(SimContractOptions),
    TestLocal,
    TestTcp,
    TestGeneric
}