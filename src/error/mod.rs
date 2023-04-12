mod gen;

pub use gen::*;

#[derive(Debug, Clone, thiserror::Error)]
pub enum BrydzSimError{
    #[error("Custom error {0}")]
    Custom(String),
    #[error("Error in game generation: {0}")]
    Gen(GenError),
}