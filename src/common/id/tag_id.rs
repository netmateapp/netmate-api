use serde::Serialize;

use crate::common::uuid::uuid4::Uuid4;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TagId(Uuid4);

impl TagId {
    pub fn value(&self) -> Uuid4 {
        self.0
    }
}

impl TagId {
    pub const fn new(value: Uuid4) -> Self {
        TagId(value)
    }
}

impl Serialize for TagId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        self.value().serialize(serializer)
    }
}