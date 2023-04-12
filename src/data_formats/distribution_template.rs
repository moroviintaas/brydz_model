use serde::{Deserialize, Serialize};
use brydz_core::sztorm::state::{ HandInfoSuspect};


#[derive(Deserialize, Serialize)]
pub enum DistributionTemplate {
    Simple,
    Suspect(HandInfoSuspect)
}