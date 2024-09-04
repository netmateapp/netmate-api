use uuid4::Uuid4;
use uuid7::Uuid7;

pub mod uuid4;
pub mod uuid7;

#[derive(Debug, Clone, PartialEq)]
pub struct AccountId(Uuid7);

impl AccountId {
    pub fn gen() -> Self {
        AccountId(Uuid7::now())
    }

    pub fn value(&self) -> &Uuid7 {
        &self.0
    }
}

impl AccountId {
    pub const fn new(value: Uuid7) -> Self {
        AccountId(value)
    }
}

pub struct TagId(Uuid4);

impl TagId {
    pub fn value(&self) -> &Uuid4 {
        &self.0
    }
}

impl TagId {
    pub const fn new(value: Uuid4) -> Self {
        TagId(value)
    }
}