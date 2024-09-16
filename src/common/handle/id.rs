use std::fmt::{self, Display};

use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::CqlValue};
use serde::{Deserialize, Serialize};

use crate::common::uuid::uuid4::Uuid4;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct HandleId(Uuid4);

impl HandleId {
    pub fn gen() -> Self {
        HandleId(Uuid4::gen())
    }

    pub const fn of(value: Uuid4) -> Self {
        HandleId(value)
    }

    pub fn value(&self) -> Uuid4 {
        self.0
    }
}

impl Display for HandleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromCqlVal<Option<CqlValue>> for HandleId {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        Uuid4::from_cql(cql_val).map(HandleId)
    }
}