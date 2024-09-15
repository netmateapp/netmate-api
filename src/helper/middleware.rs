use std::sync::Arc;

use scylla::Session;

use crate::middlewares::{manage_session::middleware::ManageSessionLayer, rate_limit::{dsl::increment_rate::{InculsiveLimit, TimeWindow}, interpreter::EndpointName, middleware::RateLimitLayer}, start_session::middleware::StartSessionLayer};

use super::{error::InitError, redis::Pool};

pub async fn session_manager<T>(db: Arc<Session>, cache: Arc<Pool>) -> Result<ManageSessionLayer, InitError<T>> {
    ManageSessionLayer::try_new(db.clone(), cache.clone())
        .await
        .map_err(|e| InitError::<T>::new(e.into()))
}

pub async fn rate_limiter<T>(db: Arc<Session>, cache: Arc<Pool>, endpoint_name: EndpointName, limit: InculsiveLimit, time_window: TimeWindow) -> Result<RateLimitLayer, InitError<T>> {
    RateLimitLayer::try_new(db.clone(), cache.clone(), endpoint_name, limit, time_window)
        .await
        .map_err(|e| InitError::<T>::new(e.into()))
}

pub async fn session_starter<T>(db: Arc<Session>, cache: Arc<Pool>) -> Result<StartSessionLayer, InitError<T>> {
    StartSessionLayer::try_new(db.clone(), cache.clone())
        .await
        .map_err(|e| InitError::<T>::new(e.into()))
}

