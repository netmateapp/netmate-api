use std::sync::Arc;

use redis::cmd;
use reqwest::{multipart::Form, Client};
use serde_json::Value;
use crate::{common::{api_key::{expiration::ApiKeyExpirationSeconds, refreshed_at::LastApiKeyRefreshedAt, key::ApiKey}, fallible::Fallible, turnstile::TurnstileToken, unixtime::UnixtimeMillis}, helper::{error::InitError, redis::{connection::{conn, Pool}, namespace::NAMESPACE_SEPARATOR, namespaces::API_KEY}}};

use super::dsl::{IssueApiKey, IssueApiKeyError};

pub struct IssueApiKeyImpl {
    cache: Arc<Pool>,
    client: Arc<Client>,
    turnstile_secret_key: String,
}

impl IssueApiKeyImpl {
    pub async fn try_new(cache: Arc<Pool>, client: Arc<Client>, turnstile_secret_key: String) -> Result<Self, InitError<Self>> {
        Ok(Self{ cache, client, turnstile_secret_key })
    }
}

impl IssueApiKey for IssueApiKeyImpl {
    async fn is_valid_token(&self, token: &TurnstileToken) -> Fallible<bool, IssueApiKeyError> {
        let form = Form::new()
            .text("secret", self.turnstile_secret_key.clone())
            .text("response", token.value().clone());

        let url = "https://challenges.cloudflare.com/turnstile/v0/siteverify";
        let response = self.client
            .post(url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| IssueApiKeyError::IsValidTokenFailed(e.into()))?;

        let outcome: Value = response.json()
            .await
            .map_err(|e| IssueApiKeyError::IsValidTokenFailed(e.into()))?;

        let success = outcome["success"].as_bool().unwrap_or(false);

        Ok(success)
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