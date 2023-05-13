use serde::{Deserialize, Serialize};
use brydz_core::sztorm::state::{CardDistribution};


#[derive(Deserialize, Serialize, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum DistributionTemplate {
    Simple,
    Suspect(CardDistribution)
}