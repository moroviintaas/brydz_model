use brydz_core::contract::ContractParameters;
use brydz_core::player::side::SideMap;
use karty::hand::CardSet;
use crate::CardDistribution;


#[derive(serde::Serialize, serde::Deserialize)]
pub struct SimContractParams {
    parameters: ContractParameters,
    info_sets: SideMap<CardDistribution>
}