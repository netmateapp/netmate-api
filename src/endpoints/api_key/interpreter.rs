use std::sync::Arc;

use redis::cmd;
use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{api_key::{expiration::ApiKeyExpirationSeconds, refreshed_at::LastApiKeyRefreshedAt, ApiKey}, fallible::Fallible, turnstile::TurnstileToken, unixtime::UnixtimeMillis}, helper::{error::InitError, redis::{connection::{conn, Pool}, namespace::NAMESPACE_SEPARATOR, namespaces::API_KEY}, scylla::prepare}};

use super::dsl::{IssueApiKey, IssueApiKeyError};

pub struct IssueApiKeyImpl {
    db: Arc<Session>,
    cache: Arc<Pool>,
    insert_api_key: Arc<PreparedStatement>,
}

impl IssueApiKeyImpl {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<Self>> {
        let insert_api_key = prepare(&db, "INSERT INTO api_keys (api_key, refreshed_at) VALUES (?, ?) USING TTL ?").await?;

        Ok(Self{ db, cache, insert_api_key })
    }
}

impl IssueApiKey for IssueApiKeyImpl {
    async fn is_valid_token(&self, token: &TurnstileToken) -> Fallible<bool, IssueApiKeyError> {

    }

    async fn try_assign_new_api_key_if_unused(&self, new_api_key: &ApiKey, expiration: ApiKeyExpirationSeconds) -> Fallible<(), IssueApiKeyError> {
        let mut conn = conn(&self.cache, |e| IssueApiKeyError::TryAssignNewApiKeyFailed(e.into())).await?;

        cmd("SET")
            .arg(format!("{}{}{}", API_KEY, NAMESPACE_SEPARATOR, new_api_key))
            .arg(LastApiKeyRefreshedAt::new(UnixtimeMillis::now()))
            .arg("NX")
            .arg("EX")
            .arg(expiration)
            .query_async::<Option<()>>(&mut *conn)
            .await
            .map_err(|e| IssueApiKeyError::TryAssignNewApiKeyFailed(e.into()))?
            .map_or_else(|| Err(IssueApiKeyError::ApiKeyAlreadyUsed), |_| Ok(()))
    }
}