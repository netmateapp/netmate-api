use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::CqlValue};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ProposalOperation {
    LowRated = 0,
    Rated = 1,
    HighRated = 2,
    Proposed = 127,
}

impl TryFrom<u8> for ProposalOperation {
    type Error = ParseProposalOperationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let operation = match value {
            0 => Self::LowRated,
            1 => Self::Rated,
            2 => Self::HighRated,
            127 => Self::Proposed,
            _ => return Err(ParseProposalOperationError),
        };
        Ok(operation)
    }
}

impl TryFrom<i8> for ProposalOperation {
    type Error = ParseProposalOperationError;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        ProposalOperation::try_from(value as u8)
    }
}

#[derive(Debug, Error)]
#[error("提案に関する操作の解析に失敗しました")]
pub struct ParseProposalOperationError;

impl FromCqlVal<Option<CqlValue>> for ProposalOperation {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        i8::from_cql(cql_val).and_then(|v| ProposalOperation::try_from(v).map_err(|_| FromCqlValError::BadVal))
    }
}