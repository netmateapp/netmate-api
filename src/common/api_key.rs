use std::{fmt::Display, str::FromStr};

use thiserror::Error;

use super::token::{calc_entropy_bytes, Token};

const API_KEY_ENTROPY_BITS: usize = 196;

type AK = Token<{calc_entropy_bytes(API_KEY_ENTROPY_BITS)}>;

#[derive(Debug, PartialEq)]
pub struct ApiKey(AK);

impl ApiKey {
    pub fn gen() -> Self {
        Self(AK::gen())
    }

    pub fn value(&self) -> &AK {
        &self.0
    }
}

impl Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
#[error("APIキーへの変換に失敗しました")]
pub struct ParseApiKeyError(#[source] pub anyhow::Error);

impl FromStr for ApiKey {
    type Err = ParseApiKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Token::from_str(s)
            .map(|t| Self(t))
            .map_err(|e| ParseApiKeyError(e.into()))
    }
}