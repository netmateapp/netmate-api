use std::{fmt::{self, Display, Formatter}, str::FromStr};

use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use thiserror::Error;

use crate::common::token::{calc_entropy_bytes, Token};

const SESSION_SERIES_ENTROPY_BITS: usize = 120;

type SS = Token<{calc_entropy_bytes(SESSION_SERIES_ENTROPY_BITS)}>;

#[derive(Debug, PartialEq)]
pub struct SessionSeries(SS);

impl SessionSeries {
    pub fn gen() -> Self {
        Self(SS::gen())
    }

    pub fn value(&self) -> &SS {
        &self.0
    }
}

impl Display for SessionSeries {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
#[error("ログイン系列識別子への変換に失敗しました")]
pub struct ParseSessionSeriesError(#[source] pub anyhow::Error);

impl FromStr for SessionSeries {
    type Err = ParseSessionSeriesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Token::from_str(s)
            .map(Self)
            .map_err(|e| ParseSessionSeriesError(e.into()))
    }
}

impl SerializeValue for SessionSeries {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        self.0.serialize(typ, writer)
    }
}

impl FromCqlVal<Option<CqlValue>> for SessionSeries {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        Token::from_cql(cql_val).map(Self)
    }
}