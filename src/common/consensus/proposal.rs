use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum IsProposal {
    Proposal,
    NotProposal,
}

impl From<IsProposal> for bool {
    fn from(value: IsProposal) -> Self {
        match value {
            IsProposal::Proposal => true,
            IsProposal::NotProposal => false,
        }
    }
}

impl From<bool> for IsProposal {
    fn from(value: bool) -> Self {
        if value {
            IsProposal::Proposal
        } else {
            IsProposal::NotProposal
        }
    }
}

impl SerializeValue for IsProposal {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&bool::from(*self), typ, writer)
    }
}

impl FromCqlVal<Option<CqlValue>> for IsProposal {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        bool::from_cql(cql_val).map(IsProposal::from)
    }
}