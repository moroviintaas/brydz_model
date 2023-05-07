mod gen;
mod simulation;

use tch::TchError;
use tensorflow::{SaveModelError, Status};
use brydz_core::error::BridgeCoreError;
use brydz_core::sztorm::spec::ContractProtocolSpec;
pub use gen::*;
pub use simulation::*;
use sztorm::error::{SetupError, SztormError};
use crate::error::BrydzSimError::Sztorm;

#[derive(Debug,  thiserror::Error)]
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
    Sztorm(SztormError<ContractProtocolSpec>),
    #[error("Tensorflow Error {0}")]
    TensorflowStatus(Status),
    #[error("SaveModel Error {0}")]
    SaveModel(SaveModelError),
    #[error("LoadModel Error {0}")]
    Tch(TchError),

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

impl From<SetupError<ContractProtocolSpec>> for BrydzSimError{
    fn from(value: SetupError<ContractProtocolSpec>) -> Self {
        Sztorm(SztormError::Setup(value))
    }
}

impl From<tensorflow::Status> for BrydzSimError{
    fn from(value: Status) -> Self {
        Self::TensorflowStatus(value)
    }
}
impl From<TchError> for BrydzSimError{
    fn from(value: TchError) -> Self {
        Self::Tch(value)
    }
}