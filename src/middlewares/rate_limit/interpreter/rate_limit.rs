use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, FromRow, Session};

use crate::{common::{api_key::ApiKey, fallible::Fallible}, helper::scylla::{Statement, TypedStatement}, middlewares::rate_limit::dsl::rate_limit::{LastApiKeyRefreshedAt, RateLimit, RateLimitError}};

use super::RateLimitImpl;


impl RateLimit for RateLimitImpl {
    // ScyllaDBのキャッシュは高速であるため問題ないが、
    // 複数のエンドポイントで同じ検証をするのは効率が悪いので、
    // 30分～1時間程度の短時間キャッシュを行うべき(リフレッシュ時刻も併せてキャッシュするため、短時間にする必要がある)
    async fn fetch_last_api_key_refreshed_at(&self, api_key: &ApiKey) -> Fallible<Option<LastApiKeyRefreshedAt>, RateLimitError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> RateLimitError {
            RateLimitError::FetchLastApiKeyRefreshedAt(e.into())
        }

        self.select_last_api_key_refreshed_at
            .query(&self.db, (api_key, ))
            .await
            .map(|o| o.map(|(refreshed_at, )| refreshed_at))
            .map_err(handle_error)
    }
}

pub const SELECT_LAST_API_KEY_REFRESHED_AT: Statement<SelectLastApiKeyRefreshedAt> = Statement::of("SELECT refreshed_at FROM api_keys WHERE api_key = ?");

#[derive(Debug)]
pub struct SelectLastApiKeyRefreshedAt(pub PreparedStatement);

impl<'a> TypedStatement<(&'a ApiKey, ), (LastApiKeyRefreshedAt, )> for SelectLastApiKeyRefreshedAt {
    type Result<U> = Option<U> where U: FromRow;

    async fn query(&self, session: &Arc<Session>, values: (&'a ApiKey, )) -> anyhow::Result<Self::Result<(LastApiKeyRefreshedAt, )>> {
        session.execute_unpaged(&self.0, values)
            .await
            .map_err(anyhow::Error::from)?
            .maybe_first_row_typed()
            .map_err(anyhow::Error::from)
    }
}