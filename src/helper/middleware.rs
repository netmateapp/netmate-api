use std::sync::Arc;

use scylla::Session;

use crate::middlewares::{manage_session::middleware::ManageSessionLayer, rate_limit::{dsl::increment_rate::{InculsiveLimit, TimeWindow}, interpreter::EndpointName, middleware::RateLimitLayer}};

use super::{error::InitError, valkey::Pool};

pub async fn session_manager<T>(db: Arc<Session>, cache: Arc<Pool>) -> Result<ManageSessionLayer, InitError<T>> {
    ManageSessionLayer::try_new(db.clone(), cache.clone())
        .await
        .map_err(|e| InitError::<T>::new(e.into()))
}

pub async fn rate_limiter<T>(db: Arc<Session>, cache: Arc<Pool>, endpoint_name: &'static str, limit: InculsiveLimit, time_window: TimeWindow) -> Result<RateLimitLayer, InitError<T>> {
    let endpoint_name = EndpointName::new(endpoint_name)
        .map_err(|e| InitError::<T>::new(e.into()))?;

    RateLimitLayer::try_new(db.clone(), cache.clone(), endpoint_name, limit, time_window)
        .await
        .map_err(|e| InitError::<T>::new(e.into()))
}

