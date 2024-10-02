use std::fmt::{self, Display};

#[derive(Debug, PartialEq)]
pub struct TurnstileToken(String);

impl TurnstileToken {
    pub fn new(token: String) -> Self {
        Self(token)
    }

    pub fn value(&self) -> &String {
        &self.0
    }
}

impl Display for TurnstileToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}