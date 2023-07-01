mod deal;
mod subtrump;
mod force_declarer;
mod choice_doubling;
use rand::distributions::{Distribution};

use std::path::{PathBuf};
pub use deal::*;
pub use subtrump::*;
pub use force_declarer::*;

use clap::Args;
use std::io::Write;
use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use brydz_core::bidding::{Bid, Doubling};
use brydz_core::cards::trump::{Trump, TrumpGen};
use brydz_core::contract::ContractParameters;
use brydz_core::deal::{BiasedHandDistribution, fair_bridge_deal};
use brydz_core::player::side::{Side};
use brydz_core::ron::ser::{PrettyConfig, to_string_pretty};
use karty::hand::CardSet;
use karty::random::RandomSymbol;
use karty::suits::Suit;
use crate::{DistributionTemplate, SimContractParams};
use crate::error::BrydzSimError;
use crate::error::GenError::LowerBoundOverUpper;
use crate::options::operation::gen_contract::choice_doubling::ChoiceDoubling;

#[derive(Args)]
pub struct GenContractOptions {
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
    /*
    #[arg(short = 'n', long = "north_type", help = "Type of North's hand information set", default_value_t = HandInfoVariants::Simple)]
    pub north_hand_type: HandInfoVariants,
    #[arg(short = 'e', long = "east_type", help = "Type of East's hand information set", default_value_t = HandInfoVariants::Simple)]
    pub east_hand_type: HandInfoVariants,
    #[arg(short = 's', long = "south_type", help = "Type of South's hand information set", default_value_t = HandInfoVariants::Simple)]
    pub south_hand_type: HandInfoVariants,
    #[arg(short = 'w', long = "west_type", help = "Type of West's hand information set", default_value_t = HandInfoVariants::Simple)]
    pub west_hand_type: HandInfoVariants,

     */
    #[arg(short = 't', long = "trump_limit", help = "Subset of possible trumps", default_value_t = Subtrump::All, rename_all = "snake_case")]
    pub trump_limit: Subtrump,
    #[arg(short = 'f', long = "force_declarer", help = "Force one side to be declarer", default_value_t = ForceDeclarer::DontForce, value_enum)]
    pub force_declarer: ForceDeclarer,
    #[arg(short = 'd', long = "doubling", help = "Force one side to be declarer", default_value_t = ChoiceDoubling::No, value_enum)]
    pub choice_doubling: ChoiceDoubling,


}

//pub fn random_contract_with_declarer(rng: &mut ThreadRng) -> Result<SimContractParams>

fn generate_single_contract(params: &GenContractOptions, rng: &mut ThreadRng) -> Result<SimContractParams, BrydzSimError>{

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
        ForceDeclarer::DontForce => Side::random(rng),
        _ => Side::try_from(&params.force_declarer).unwrap(),
    };



    let doubling = match params.choice_doubling{
        ChoiceDoubling::Any => *[Doubling::None, Doubling::Redouble, Doubling::Redouble].choose(rng).unwrap(),
        ChoiceDoubling::No => Doubling::None,
        ChoiceDoubling::Double => Doubling::Double,
        ChoiceDoubling::Redouble => Doubling::Redouble,
    };
    let contract_parameters = ContractParameters::new_d(contract_declarer,
                                        Bid::init(trump, contract_value).unwrap(),
                                        doubling);


    let (template, cards) = match params.deal_method{
        DealMethod::Fair => (DistributionTemplate::Simple, fair_bridge_deal::<CardSet>()),

        DealMethod::Biased => {
            let mut rng = thread_rng();
            let distribution: BiasedHandDistribution = rng.gen();
            let cards = distribution.sample(&mut rng);
            (DistributionTemplate::Suspect(distribution), cards)
        }
    };

    Ok(SimContractParams::new(contract_parameters, template, cards))


}

pub fn generate_contracts(params: &GenContractOptions) -> Result<Vec<SimContractParams>, BrydzSimError>{
    let repeat = params.game_count as usize;
    let mut rng = thread_rng();
    let mut game_params: Vec<SimContractParams> = Vec::with_capacity(repeat);
    for _ in 0..repeat{
        game_params.push(generate_single_contract(params, &mut rng)?);
    }
    Ok(game_params)

}

pub fn gen2(gen_options: &GenContractOptions) -> Result<(), BrydzSimError>{
    let my_config = PrettyConfig::new()
        .depth_limit(4)
        // definitely superior (okay, just joking)
        .indentor("\t".to_owned());
    let contracts = generate_contracts(gen_options).unwrap();

    match &gen_options.output_file{
        None => {
            println!("{}", to_string_pretty(&contracts, my_config).unwrap())
        }
        Some(file) => {
            let mut output = std::fs::File::create(file).unwrap();
            write!(output, "{}", to_string_pretty(&contracts, my_config).unwrap()).unwrap()
        }
    };
    Ok(())
}