use uuid4::Uuid4;
use uuid7::Uuid7;

pub mod uuid4;
pub mod uuid7;

pub type AccountId = Uuid7;
pub type TagId = Uuid4;