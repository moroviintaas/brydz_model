use brydz_core::contract::ContractParametersGen;
use brydz_core::player::side::SideMap;
use karty::hand::CardSet;

pub struct SimContractParams {
    parameters: ContractParameters,
    card_distribution: SideMap<CardSet>
}