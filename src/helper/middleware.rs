use std::sync::Arc;

use scylla::Session;

use crate::middlewares::{limit::{Count, EndpointName, InculsiveLimit, TimeUnit}, manage_session::middleware::ManageSessionLayer, quota_limit::middleware::QuotaLimitLayer, rate_limit::middleware::RateLimitLayer, start_session::middleware::StartSessionLayer};

use super::{error::InitError, redis::{namespace::Namespace, connection::Pool}};

pub async fn session_manager<T>(db: Arc<Session>, cache: Arc<Pool>) -> Result<ManageSessionLayer, InitError<T>> {
    ManageSessionLayer::try_new(db, cache)
        .await
        .map_err(|e| InitError::<T>::new(e.into()))
}

pub async fn rate_limiter<T>(db: Arc<Session>, cache: Arc<Pool>, endpoint_name: &'static str, limit: u32, time_window: u32, time_unit: TimeUnit) -> Result<RateLimitLayer, InitError<T>> {
    let endpoint_name = EndpointName::new(Namespace::of(endpoint_name));
    let limit = InculsiveLimit::new(Count::new(limit));

    RateLimitLayer::try_new(db, cache, endpoint_name, limit, time_unit.apply(time_window))
        .await
        .map_err(|e| InitError::<T>::new(e.into()))
}

pub async fn session_starter<T>(db: Arc<Session>, cache: Arc<Pool>) -> Result<StartSessionLayer, InitError<T>> {
    StartSessionLayer::try_new(db, cache)
        .await
        .map_err(|e| InitError::<T>::new(e.into()))
}

pub async fn quota_limiter<T>(db: Arc<Session>, cache: Arc<Pool>, endpoint_name: &'static str, time_window: u32, time_unit: TimeUnit) -> Result<QuotaLimitLayer, InitError<T>> {
    let endpoint_name = EndpointName::new(Namespace::of(endpoint_name));

    QuotaLimitLayer::try_new(db, cache, endpoint_name, time_unit.apply(time_window))
        .await
        .map_err(|e| InitError::<T>::new(e.into()))
}

