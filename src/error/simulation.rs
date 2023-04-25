use brydz_core::sztorm::spec::ContractProtocolSpec;
use sztorm::error::SetupError;
use crate::error::BrydzSimError;

#[derive(thiserror::Error, Debug, Clone)]
pub enum SimulationError {
    #[error("Wrong ContractParams: {0}")]
    WrongContractParams(String),
    //#[error("Error setting up sztorm model: {0}")]
    //SztormSetup(sztorm::error::SetupError<ContractProtocolSpec>)


}

impl From<SimulationError> for BrydzSimError{
    fn from(value: SimulationError) -> Self {
        Self::Simulation(value)
    }
}
/*
impl From<sztorm::error::SetupError<ContractProtocolSpec>> for SimulationError{
    fn from(value: SetupError<ContractProtocolSpec>) -> Self {
        Self::Setup(value)
    }
}

 */