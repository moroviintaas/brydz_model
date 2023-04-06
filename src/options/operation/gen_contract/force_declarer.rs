use clap::ValueEnum;
use brydz_core::player::side::Side;
use crate::error::BrydzSimError;
use crate::error::GenError::ConvForceDeclarerNoToSide;

#[derive(ValueEnum, Clone, Debug)]
pub enum ForceDeclarer{
    No,
    North,
    East,
    South,
    West
}

impl TryFrom<&ForceDeclarer> for Side{
    type Error = BrydzSimError;

    fn try_from(value: &ForceDeclarer) -> Result<Self, Self::Error> {
        match value{
            ForceDeclarer::No => Err(BrydzSimError::Gen(ConvForceDeclarerNoToSide)),
            ForceDeclarer::North => Ok(Side::North),
            ForceDeclarer::East => Ok(Side::East),
            ForceDeclarer::South => Ok(Side::South),
            ForceDeclarer::West => Ok(Side::West)
        }
    }
}