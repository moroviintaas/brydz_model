use serde::{Deserialize, Serialize};
use brydz_core::sztorm::state::{ContractAgentInfoSetSimple, ContractDummyState, HandInfoSimple, HandInfoSuspect};
use karty::hand::CardSet;


#[derive(Deserialize, Serialize)]
pub enum CardDistribution {
    Simple(HandInfoSimple),
    Suspect(HandInfoSuspect)
}