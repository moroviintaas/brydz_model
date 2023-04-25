use std::path::PathBuf;
use clap::Args;
use crate::error::BrydzSimError;

#[derive(Args)]
pub struct SimContractOptions{

    #[arg(short = 'r', long = "repeat", help = "Repeat each contract a number of times", default_value = "1")]
    pub game_count: u16,
    #[arg(short = 'i', long = "input", help = "File with contracts to play")]
    pub input_file: Option<PathBuf>

}



pub(crate) fn sim2(gen_options: &SimContractOptions) -> Result<(), BrydzSimError>{
    todo!()
}