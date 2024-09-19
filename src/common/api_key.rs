use std::{fmt::Display, str::FromStr};

use scylla::{frame::response::result::ColumnType, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use thiserror::Error;

use super::token::{calc_entropy_bytes, Token};

const API_KEY_ENTROPY_BITS: usize = 196;

type AK = Token<{calc_entropy_bytes(API_KEY_ENTROPY_BITS)}>;

#[derive(Debug, PartialEq)]
pub struct ApiKey(AK);

impl ApiKey {
    pub fn gen() -> Self {
        Self(AK::gen())
    }

    pub fn value(&self) -> &AK {
        &self.0
    }
}

impl Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
#[error("APIキーへの変換に失敗しました")]
pub struct ParseApiKeyError(#[source] pub anyhow::Error);

impl FromStr for ApiKey {
    type Err = ParseApiKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Token::from_str(s)
            .map(Self)
            .map_err(|e| ParseApiKeyError(e.into()))
    }
}

impl SerializeValue for ApiKey {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        self.value().serialize(typ, writer)
    }
}