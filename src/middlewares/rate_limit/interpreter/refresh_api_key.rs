use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, FromRow, Session};

use crate::{common::{api_key::ApiKey, fallible::Fallible, unixtime::UnixtimeMillis}, helper::scylla::{Statement, TypedStatement, Unit}, middlewares::rate_limit::dsl::refresh_api_key::{ApiKeyExpirationSeconds, ApiKeyRefreshThereshold, RefreshApiKey, RefreshApiKeyError}};

use super::RateLimitImpl;

const API_KEY_REFRESH_THERESHOLD: ApiKeyRefreshThereshold = ApiKeyRefreshThereshold::days(10);
const API_KEY_EXPIRATION: ApiKeyExpirationSeconds = ApiKeyExpirationSeconds::secs(2592000);

impl RefreshApiKey for RateLimitImpl {
    fn api_key_refresh_thereshold(&self) -> ApiKeyRefreshThereshold {
        API_KEY_REFRESH_THERESHOLD
    }

    fn api_key_expiration(&self) -> ApiKeyExpirationSeconds {
        API_KEY_EXPIRATION
    }

    async fn refresh_api_key(&self, api_key: &ApiKey, expiration: ApiKeyExpirationSeconds) -> Fallible<(), RefreshApiKeyError> {
        self.insert_api_key_with_ttl_refresh
            .execute(&self.db, (api_key, UnixtimeMillis::now(), expiration))
            .await
            .map_err(|e| RefreshApiKeyError::RefreshApiKeyFailed(e.into()))
    }
}

pub const INSERT_API_KEY_WITH_TTL_REFRESH: Statement<InsertApiKeyWithTtlRefresh>
    = Statement::of("INSERT INTO api_keys (api_key, refreshed_at) VALUES (?, ?) USING TTL ?");

#[derive(Debug)]
pub struct InsertApiKeyWithTtlRefresh(pub PreparedStatement);

impl<'a> TypedStatement<(&'a ApiKey, UnixtimeMillis, ApiKeyExpirationSeconds), Unit> for InsertApiKeyWithTtlRefresh {
    type Result<U> = U where U: FromRow;

    async fn query(&self, session: &Arc<Session>, values: (&'a ApiKey, UnixtimeMillis, ApiKeyExpirationSeconds)) -> anyhow::Result<Unit> {
        session.execute_unpaged(&self.0, values)
            .await
            .map(|_| Unit)
            .map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::helper::scylla::check_cql_statement_type;

    use super::INSERT_API_KEY_WITH_TTL_REFRESH;

    #[test]
    fn check_insert_api_key_with_ttl_refresh_type() {
        check_cql_statement_type(INSERT_API_KEY_WITH_TTL_REFRESH);
    }
}