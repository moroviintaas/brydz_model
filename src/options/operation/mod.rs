mod gen_contract;
pub mod test_ops;


pub use gen_contract::*;
pub use clap::Subcommand;

#[derive(Subcommand)]
pub enum Operation {

    Gen2(GenContractOptions),
    TestLocal,
    TestTcp,
}