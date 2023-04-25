mod gen;
mod simulation;

use brydz_core::error::BridgeCoreError;
use brydz_core::sztorm::spec::ContractProtocolSpec;
pub use gen::*;
pub use simulation::*;
use sztorm::error::{SetupError, SztormError};
use crate::error::BrydzSimError::Sztorm;

#[derive(Debug, Clone, thiserror::Error)]
pub enum BrydzSimError{
    #[error("Custom error {0}")]
    Custom(String),
    #[error("Error in game generation: {0}")]
    Gen(GenError),
    #[error("Error in game setup: {0}")]
    Simulation(SimulationError),
    //#[error("Error during playing game: {0}")]
    //Game(BridgeCoreError),
    #[error("Error in Sztorm framework: {0}")]
    Sztorm(SztormError<ContractProtocolSpec>)

}

impl From<BridgeCoreError> for BrydzSimError{
    fn from(value: BridgeCoreError) -> Self {
        Self::Sztorm(SztormError::Game(value))
    }
}


impl From<SztormError<ContractProtocolSpec>> for BrydzSimError{
    fn from(value: SztormError<ContractProtocolSpec>) -> Self {
        Self::Sztorm(value)
    }
}
