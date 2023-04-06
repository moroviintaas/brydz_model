mod deal;
mod subtrump;
mod force_declarer;

use std::path::{Path, PathBuf};
pub use deal::*;
pub use subtrump::*;
pub use force_declarer::*;

use clap::Args;
use rand::Rng;
use rand::rngs::ThreadRng;
use brydz_core::cards::trump::{Trump, TrumpGen};
use brydz_core::contract::ContractParameters;
use brydz_core::player::side::{Side, SIDES};
use karty::random::RandomSymbol;
use karty::suits::Suit;
use crate::SimContractParams;
use crate::error::BrydzSimError;
use crate::error::GenError::LowerBoundOverUpper;
use crate::options::hand_info::HandInfoVariants;

#[derive(Args)]
pub struct GenContract {
    #[arg(short = 'g', long = "game_count", help = "Number of game parameters to generate", default_value = "1")]
    pub game_count: u16,
    #[arg(short = 'm', long = "method", value_enum,  help = "Probability method of distribution cards", default_value_t = DealMethod::Fair)]
    pub deal_method: DealMethod,
    #[arg(short = 'l', long = "lower_bound", help = "Minimal contract value", value_parser = clap::value_parser!(u8).range(1..=7), default_value = "1")]
    pub min_contract: u8,
    #[arg(short = 'u', long = "upper_bound", help = "Maximal contract value", value_parser = clap::value_parser!(u8).range(1..=7), default_value = "7")]
    pub max_contract: u8,
    #[arg(short = 'o', long = "output", help = "Path to output file")]
    pub output_file: Option<PathBuf>,
    #[arg(short = 'p', long = "probability_file", help = "Path to file with probabilities of cards")]
    pub probability_file: Option<PathBuf>,
    #[arg(short = 'n', long = "north_type", help = "Type of North's hand information set", default_value_t = HandInfoVariants::Simple)]
    pub north_hand_type: HandInfoVariants,
    #[arg(short = 'e', long = "east_type", help = "Type of East's hand information set", default_value_t = HandInfoVariants::Simple)]
    pub east_hand_type: HandInfoVariants,
    #[arg(short = 's', long = "south_type", help = "Type of South's hand information set", default_value_t = HandInfoVariants::Simple)]
    pub south_hand_type: HandInfoVariants,
    #[arg(short = 'w', long = "west_type", help = "Type of West's hand information set", default_value_t = HandInfoVariants::Simple)]
    pub west_hand_type: HandInfoVariants,
    #[arg(short = 't', long = "trump_limit", help = "Subset of possible trumps", default_value_t = Subtrump::All, rename_all = "snake_case")]
    pub trump_limit: Subtrump,
    #[arg(short = 'f', long = "force_declarer", help = "Force one side to be declarer", default_value_t = ForceDeclarer::No, value_enum)]
    pub force_declarer: ForceDeclarer


}

fn generate_single_contract(params: &GenContract, rng: &mut ThreadRng) -> Result<(), BrydzSimError>{

    if params.min_contract > params.max_contract {
        return Err(BrydzSimError::Gen(LowerBoundOverUpper {lower: params.min_contract, upper: params.max_contract }))
    }

    let contract_value = rng.gen_range(params.min_contract..=params.max_contract);
    let trump: Trump = match params.trump_limit{
        Subtrump::All => TrumpGen::<Suit>::random(rng),
        Subtrump::Colored => {
            Trump::Colored(Suit::random(rng))
        },
        Subtrump::NoTrump => Trump::NoTrump
    };
    let contract_declarer: Side = match params.force_declarer {
        ForceDeclarer::No => Side::random(rng),
        _ => Side::try_from(&params.force_declarer).unwrap(),
    };

    //let contract_params = ContractParameters::new(contract_declarer)



    match params.deal_method{
        DealMethod::Fair => {
            //let deal
        }
    }
    todo!();
    Ok(())
}

pub fn generate_contracts(params: &GenContract) -> Result<(), BrydzSimError>{
    let repeat = params.game_count as usize;
    let mut game_params: Vec<SimContractParams> = Vec::with_capacity(repeat);
    for _ in 0..repeat{

    }


    todo!()
}