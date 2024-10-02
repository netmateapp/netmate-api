use thiserror::Error;

use crate::common::{api_key::{expiration::ApiKeyExpirationSeconds, key::ApiKey, API_KEY_EXPIRATION}, fallible::Fallible, turnstile::TurnstileToken};

pub(crate) trait IssueApiKey {
    async fn issue_api_key(&self, token: &TurnstileToken) -> Fallible<ApiKey, IssueApiKeyError> {
        if self.is_valid_token(token).await? {
            let new_api_key = self.assign_new_api_key_if_unused().await?;
            Ok(new_api_key)
        } else {
            Err(IssueApiKeyError::InvalidToken)
        }
    }

    fn api_key_expiration(&self) -> ApiKeyExpirationSeconds {
        API_KEY_EXPIRATION
    }

    async fn is_valid_token(&self, token: &TurnstileToken) -> Fallible<bool, IssueApiKeyError>;

    async fn assign_new_api_key_if_unused(&self) -> Fallible<ApiKey, IssueApiKeyError> {
        let mut new_api_key = ApiKey::gen();

        // 奇跡が起きない限りO(1)で終わる
        loop {
            match self.try_assign_new_api_key_if_unused(&new_api_key, self.api_key_expiration()).await {
                Ok(()) => return Ok(new_api_key),
                Err(IssueApiKeyError::ApiKeyAlreadyUsed) => new_api_key = ApiKey::gen(),
                Err(e) => return Err(e)
            }
        }
    }

    async fn try_assign_new_api_key_if_unused(&self, new_api_key: &ApiKey, expiration: ApiKeyExpirationSeconds) -> Fallible<(), IssueApiKeyError>;
}

#[derive(Debug, Error)]
pub enum IssueApiKeyError {
    #[error("トークンの検証に失敗しました")]
    IsValidTokenFailed(#[source] anyhow::Error),
    #[error("無効なトークンです")]
    InvalidToken,
    #[error("既に使用されているAPIキーです")]
    ApiKeyAlreadyUsed,
    #[error("APIキーの割当の試行に失敗しました")]
    TryAssignNewApiKeyFailed(#[source] anyhow::Error),
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use crate::common::{api_key::{expiration::ApiKeyExpirationSeconds, key::ApiKey}, fallible::Fallible, turnstile::TurnstileToken};

    use super::{IssueApiKey, IssueApiKeyError};

    struct MockIssueApiKey;

    static INVALID_TOKEN: LazyLock<TurnstileToken> = LazyLock::new(|| TurnstileToken::new("0".to_string()));

    impl IssueApiKey for MockIssueApiKey {
        async fn is_valid_token(&self, token: &TurnstileToken) -> Fallible<bool, IssueApiKeyError> {
            if token == &*INVALID_TOKEN {
                Ok(false)
            } else {
                Ok(true)
            }
        }

        async fn try_assign_new_api_key_if_unused(&self, _: &ApiKey, _: ApiKeyExpirationSeconds) -> Fallible<(), IssueApiKeyError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn valid_token() {
        assert!(MockIssueApiKey.issue_api_key(&TurnstileToken::new("1".to_string())).await.is_ok());
    }

    #[tokio::test]
    async fn invalid_token() {
        assert!(matches!(MockIssueApiKey.issue_api_key(&INVALID_TOKEN).await.err().unwrap(), IssueApiKeyError::InvalidToken));
    }
}