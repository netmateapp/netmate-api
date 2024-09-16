use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::CqlValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct HandleShareCount(u32);

impl HandleShareCount {
    pub const fn of(value: u32) -> Self {
        HandleShareCount(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl From<i32> for HandleShareCount {
    fn from(value: i32) -> Self {
        HandleShareCount(value as u32)
    }
}

impl FromCqlVal<Option<CqlValue>> for HandleShareCount {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        i32::from_cql(cql_val).map(HandleShareCount::from)
    }
}