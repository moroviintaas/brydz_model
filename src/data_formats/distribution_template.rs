use serde::{Deserialize, Serialize};
use brydz_core::deal::BiasedHandDistribution;


#[derive(Deserialize, Serialize, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum DistributionTemplate {
    Simple,
    Suspect(BiasedHandDistribution)
}