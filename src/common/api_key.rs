use std::str::FromStr;

use thiserror::Error;

use super::token::{calc_entropy_bytes, Token};

const NO_ACCOUNT_USER_API_KEY_ENTROPY_BITS: usize = 200;

type AK = Token<{calc_entropy_bytes(NO_ACCOUNT_USER_API_KEY_ENTROPY_BITS)}>;

pub struct ApiKey(AK);

impl ApiKey {
    pub fn gen() -> Self {
        Self(AK::gen())
    }

    pub fn value(&self) -> &AK {
        &self.0
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