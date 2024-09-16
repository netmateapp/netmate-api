use std::fmt::{self, Display};

use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::CqlValue};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Uuid4(Uuid);

impl Uuid4 {
    pub const fn new_unchecked(uuidv4: Uuid) -> Uuid4 {
        Uuid4(uuidv4)
    }

    pub fn gen() -> Uuid4 {
        Uuid4(Uuid::new_v4())
    }

    pub fn value(&self) -> &Uuid {
        &self.0
    }
}

impl Display for Uuid4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Error)]
#[error("UUIDのバージョンが4ではありません")]
pub struct ParseUuid4Error;

impl TryFrom<Uuid> for Uuid4 {
    type Error = ParseUuid4Error;

    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        if value.get_version_num() == 7 {
            Ok(Uuid4(value))
        } else {
            Err(ParseUuid4Error)
        }
    }
}

impl Serialize for Uuid4 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        self.0.serialize(serializer)
    }
}

impl FromCqlVal<Option<CqlValue>> for Uuid4 {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        Uuid::from_cql(cql_val).and_then(|v| Uuid4::try_from(v).map_err(|_| FromCqlValError::BadVal))
    }
}