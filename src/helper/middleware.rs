use std::sync::Arc;

use scylla::Session;

use crate::middlewares::{manage_session::middleware::ManageSessionLayer, rate_limit::{dsl::increment_rate::{InculsiveLimit, TimeWindow}, interpreter::EndpointName, middleware::RateLimitLayer}, start_session::middleware::StartSessionLayer};

use super::{error::InitError, redis::{Namespace, Pool}};

pub async fn session_manager<T>(db: Arc<Session>, cache: Arc<Pool>) -> Result<ManageSessionLayer, InitError<T>> {
    ManageSessionLayer::try_new(db.clone(), cache.clone())
        .await
        .map_err(|e| InitError::<T>::new(e.into()))
}

pub async fn rate_limiter<T>(db: Arc<Session>, cache: Arc<Pool>, endpoint_name: &'static str, limit: u32, time_window: u32, time_unit: TimeUnit) -> Result<RateLimitLayer, InitError<T>> {
    let endpoint_name = EndpointName::new(Namespace::of(endpoint_name));
    let limit = InculsiveLimit::new(limit);
    let time_window = match time_unit {
        TimeUnit::SECS => TimeWindow::seconds(time_window),
        TimeUnit::MINS => TimeWindow::minutes(time_window),
        TimeUnit::HOURS => TimeWindow::hours(time_window),
        TimeUnit::DAYS => TimeWindow::days(time_window),
    };

    RateLimitLayer::try_new(db.clone(), cache.clone(), endpoint_name, limit, time_window)
        .await
        .map_err(|e| InitError::<T>::new(e.into()))
}

pub enum TimeUnit {
    SECS,
    MINS,
    HOURS,
    DAYS,
}

pub async fn session_starter<T>(db: Arc<Session>, cache: Arc<Pool>) -> Result<StartSessionLayer, InitError<T>> {
    StartSessionLayer::try_new(db.clone(), cache.clone())
        .await
        .map_err(|e| InitError::<T>::new(e.into()))
}

