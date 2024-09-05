use std::str::FromStr;

use thiserror::Error;

use super::token::{calc_entropy_bytes, Token};

const NO_ACCOUNT_USER_API_KEY_ENTROPY_BITS: usize = 120;

type NAUAKey = Token<{calc_entropy_bytes(NO_ACCOUNT_USER_API_KEY_ENTROPY_BITS)}>;

pub struct NoAccountUserApiKey(NAUAKey);

impl NoAccountUserApiKey {
    pub fn gen() -> Self {
        Self(NAUAKey::gen())
    }

    pub fn value(&self) -> &NAUAKey {
        &self.0
    }
}

#[derive(Debug, Error)]
#[error("未認証APIキーへの変換に失敗しました")]
pub struct ParseNoAccountUserApiKeyError(#[source] pub anyhow::Error);

impl FromStr for NoAccountUserApiKey {
    type Err = ParseNoAccountUserApiKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Token::from_str(s)
            .map(|t| Self(t))
            .map_err(|e| ParseNoAccountUserApiKeyError(e.into()))
    }
}