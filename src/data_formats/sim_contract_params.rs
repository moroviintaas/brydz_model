use brydz_core::contract::ContractParameters;
use brydz_core::player::side::SideMap;
use karty::hand::CardSet;
use crate::DistributionTemplate;


#[derive(serde::Serialize, serde::Deserialize)]
pub struct SimContractParams {
    parameters: ContractParameters,
    //info_sets: SideMap<DistributionTemplate>
    distribution_template: DistributionTemplate,
    cards: SideMap<CardSet>

}

impl SimContractParams{
    pub fn new(parameters: ContractParameters,
               distribution_template: DistributionTemplate,
               cards: SideMap<CardSet>) -> Self{
        Self{parameters, distribution_template, cards}
    }

    pub fn cards(&self) -> &SideMap<CardSet>{
        &self.cards
    }
    pub fn parameters(&self) -> &ContractParameters{
        &self.parameters
    }
}