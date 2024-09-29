use std::sync::Arc;

use redis::{cmd, Script};
use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{fallible::Fallible, profile::account_id::AccountId}, helper::{error::InitError, redis::{conn, Namespace, Pool, NAMESPACE_SEPARATOR}, scylla::prepare}, middlewares::limit::{EndpointName, InculsiveLimit, TimeWindow}};

use super::dsl::{ConsumedQuota, QuotaLimit, QuotaLimitError};

const QUOTA_LIMIT_NAMESPACE: Namespace = Namespace::of("qtlim");

#[derive(Debug)]
pub struct QuotaLimitImpl {
    db: Arc<Session>,
    cache: Arc<Pool>,
    endpoint_name: EndpointName,
    time_window: TimeWindow,
    select_quota_limit: Arc<PreparedStatement>,
    incr_and_expire_if_first: Arc<Script>,
}

impl QuotaLimitImpl {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>, endpoint_name: EndpointName, time_window: TimeWindow) -> Result<Self, InitError<Self>> {
        let select_quota_limit = prepare(&db, "").await?;

        let incr_and_expire_if_first = Arc::new(Script::new(include_str!("incr_and_expire_if_first.lua")));

        Ok(Self { db, cache, endpoint_name, time_window, select_quota_limit, incr_and_expire_if_first })
    }
}

impl QuotaLimit for QuotaLimitImpl {
    async fn fetch_personal_limit(&self, account_id: AccountId) -> Fallible<Option<InculsiveLimit>, QuotaLimitError> {
        self.db
            .execute_unpaged(&self.select_quota_limit, (account_id, ))
            .await
            .map_err(|e| QuotaLimitError::FetchPersonalLimitFailed(e.into()))?
            .maybe_first_row_typed::<(InculsiveLimit, )>()
            .map_err(|e| QuotaLimitError::FetchPersonalLimitFailed(e.into()))
            .map(|o| o.map(|(personal_limit, )| personal_limit))
    }

    async fn fetch_consumed_quota(&self, account_id: AccountId) -> Fallible<Option<ConsumedQuota>, QuotaLimitError> {
        let mut conn = conn(&self.cache, |e| QuotaLimitError::FetchConsumedQuotaFailed(e.into())).await?;
        
        cmd("GET")
            .arg(format!("{}{}{}{}{}", QUOTA_LIMIT_NAMESPACE, NAMESPACE_SEPARATOR, self.endpoint_name, NAMESPACE_SEPARATOR, account_id))
            .query_async::<Option<ConsumedQuota>>(&mut *conn)
            .await
            .map_err(|e| QuotaLimitError::FetchConsumedQuotaFailed(e.into()))
    }

    async fn increment_consumed_quota(&self, account_id: AccountId, time_window: TimeWindow) -> Fallible<(), QuotaLimitError> {
        let mut conn = conn(&self.cache, |e| QuotaLimitError::IncrementConsumedQuotaFailed(e.into())).await?;
        
        self.incr_and_expire_if_first
            .key(format!("{}{}{}{}{}", QUOTA_LIMIT_NAMESPACE, NAMESPACE_SEPARATOR, self.endpoint_name, NAMESPACE_SEPARATOR, account_id))
            .arg(time_window)
            .invoke_async(&mut *conn)
            .await
            .map_err(|e| QuotaLimitError::IncrementConsumedQuotaFailed(e.into()))
    }

    fn time_window(&self) -> TimeWindow {
        self.time_window
    }
}