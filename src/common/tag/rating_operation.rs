use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::CqlValue};
use thiserror::Error;

pub enum RatingOperation {
    Low = 0,
    Middle = 1,
    High = 2,
    Proposed = 127,
}

impl TryFrom<u8> for RatingOperation {
    type Error = ParseRatingOperationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Low),
            1 => Ok(Self::Middle),
            2 => Ok(Self::High),
            127 => Ok(Self::Proposed),
            _ => Err(ParseRatingOperationError),
        }
    }
}

impl TryFrom<i8> for RatingOperation {
    type Error = ParseRatingOperationError;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        RatingOperation::try_from(value as u8)
    }
}

#[derive(Debug, Error)]
#[error("評価操作の解析に失敗しました")]
pub struct ParseRatingOperationError;

impl FromCqlVal<Option<CqlValue>> for RatingOperation {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        i8::from_cql(cql_val).and_then(|v| RatingOperation::try_from(v).map_err(|_| FromCqlValError::BadVal))
    }
}