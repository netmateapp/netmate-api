use std::fmt::{self, Display};

use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ZeroBasedPage(u32);

impl ZeroBasedPage {
    pub fn value(&self) -> u32 {
        self.0
    }

    pub fn first_index(&self, page_size: u32) -> u32 {
        self.0 * page_size
    }

    pub fn last_index(&self, page_size: u32) -> u32 {
        self.first_index(page_size) + page_size - 1
    }
}

impl From<u32> for ZeroBasedPage {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Display for ZeroBasedPage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'de> Deserialize<'de> for ZeroBasedPage {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u32::deserialize(deserializer).map(ZeroBasedPage::from)
    }
}