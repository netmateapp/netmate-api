use std::{fmt::{self, Display, Formatter}, str::FromStr};

use thiserror::Error;

use crate::common::token::{calc_entropy_bytes, Token};

const SESSION_ID_ENTROPY_BITS: usize = 120;

type SId = Token<{calc_entropy_bytes(SESSION_ID_ENTROPY_BITS)}>;

#[derive(Debug, PartialEq)]
pub struct SessionId(SId);

impl SessionId {
    pub fn gen() -> Self {
        Self(SId::gen())
    }

    pub fn value(&self) -> &SId {
        &self.0
    }
}

impl Display for SessionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
#[error("セッション識別子への変換に失敗しました")]
pub struct ParseSessionIdError(#[source] pub anyhow::Error);

impl FromStr for SessionId {
    type Err = ParseSessionIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Token::from_str(s)
            .map(|t| Self(t))
            .map_err(|e| ParseSessionIdError(e.into()))
    }
}