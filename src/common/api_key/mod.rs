use expiration::ApiKeyExpirationSeconds;

use crate::middlewares::rate_limit::dsl::refresh_api_key::ApiKeyRefreshThereshold;

pub mod expiration;
pub mod key;
pub mod refreshed_at;

pub const API_KEY_REFRESH_THERESHOLD: ApiKeyRefreshThereshold = ApiKeyRefreshThereshold::days(10);
pub const API_KEY_EXPIRATION: ApiKeyExpirationSeconds = ApiKeyExpirationSeconds::secs(2592000);