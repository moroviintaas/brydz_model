mod gen;
mod simulation;

use ron::Error;
use tch::TchError;
//use tensorflow::{SaveModelError, Status};
use brydz_core::error::BridgeCoreError;
use brydz_core::sztorm::spec::ContractDP;
pub use gen::*;
pub use simulation::*;
use sztorm::error::{SetupError, SztormError};
use sztorm_rl::error::SztormRLError;
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
    Sztorm(SztormError<ContractDP>),
    #[error("Error in Sztorm Reinforcement Learning framework: {0}")]
    SztormRL(SztormRLError<ContractDP>),
    //#[error("Tensorflow Error {0}")]
    //TensorflowStatus(Status),
    //#[error("SaveModel Error {0}")]
    //SaveModel(SaveModelError),
    #[error("LoadModel Error {0}")]
    Tch(TchError),
    #[error("Ron Error {0}")]
    Ron(ron::error::Error),
    #[error("IO Error {0}")]
    IO(std::io::Error),

}

impl From<BridgeCoreError> for BrydzSimError{
    fn from(value: BridgeCoreError) -> Self {
        Self::Sztorm(SztormError::Game(value))
    }
}


impl From<SztormError<ContractDP>> for BrydzSimError{
    fn from(value: SztormError<ContractDP>) -> Self {
        Self::Sztorm(value)
    }
}
impl From<SztormRLError<ContractDP>> for BrydzSimError{
    fn from(value: SztormRLError<ContractDP>) -> Self {
        Self::SztormRL(value)
    }
}

impl From<SetupError<ContractDP>> for BrydzSimError{
    fn from(value: SetupError<ContractDP>) -> Self {
        Sztorm(SztormError::Setup(value))
    }
}
impl From<ron::error::Error> for BrydzSimError{
    fn from(value: Error) -> Self {
        BrydzSimError::Ron(value)
    }
}
impl From<std::io::Error> for BrydzSimError{
    fn from(value: std::io::Error) -> Self {
        BrydzSimError::IO(value)
    }
}
/*
impl From<tensorflow::Status> for BrydzSimError{
    fn from(value: Status) -> Self {
        Self::TensorflowStatus(value)
    }
}

 */
impl From<TchError> for BrydzSimError{
    fn from(value: TchError) -> Self {
        Self::Tch(value)
    }
}