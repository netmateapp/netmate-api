
use crate::common::{fallible::Fallible, id::AccountId, session::value::{RefreshToken, SessionId, SessionSeries}};

pub struct SessionExpirationTime(u32);

impl SessionExpirationTime {
    pub const fn new(seconds: u32) -> Self {
        Self(seconds)
    }

    pub fn as_secs(&self) -> u32 {
        self.0
    }
}

pub(crate) trait UpdateSession {
    async fn update_session(&self, session_account_id: &AccountId, new_expiration: &SessionExpirationTime) -> Fallible<SessionId, UpdateSessionError> {
        let mut new_session_id = SessionId::gen();
        
        // このループは奇跡が起きない限りO(1)となる
        loop {
            match self.try_activate_new_session_id_with_expiration(&new_session_id, session_account_id, new_expiration).await {
                Ok(()) => return Ok(new_session_id),
                Err(UpdateSessionError::SessionIdAlreadyUsed) => new_session_id = SessionId::gen(),
                _ => return Err(UpdateSessionError::IssueNewSessionIdFailed)
            }
        }
    }

    async fn try_activate_new_session_id_with_expiration(&self, new_session_id: &SessionId, session_account_id: &AccountId, new_expiration: &SessionExpirationTime) -> Fallible<(), UpdateSessionError>;
}

pub enum UpdateSessionError {
    SessionIdAlreadyUsed,
    IssueNewSessionIdFailed,
}

pub struct RefreshTokenExpirationTime(u32);

impl RefreshTokenExpirationTime {
    pub const fn new(seconds: u32) -> Self {
        Self(seconds)
    }

    pub fn as_secs(&self) -> u32 {
        self.0
    }
}

pub(crate) trait UpdateRefreshToken {
    async fn update_refresh_token(&self, session_series: &SessionSeries, account_id: &AccountId, expiration: &RefreshTokenExpirationTime) -> Fallible<RefreshToken, UpdateRefreshTokenError> {
        let new_refresh_token = RefreshToken::gen();
        self.active_new_refresh_token_with_expiration(&new_refresh_token, &session_series, &account_id, expiration).await?;
        Ok(new_refresh_token)
    }

    async fn active_new_refresh_token_with_expiration(&self, new_refresh_token: &RefreshToken, session_series: &SessionSeries, session_account_id: &AccountId, expiration: &RefreshTokenExpirationTime) -> Fallible<(), UpdateRefreshTokenError>;
    /*
            // `series_id_update_at`は実際にはDBアクセスとなるため、
        // 正常にセッション管理識別子が発行されている時 = 次のアクセスが最短でも30分後である高い保証がある場合のみ処理する
        // ユーザーが意図的にセッション管理クッキーを削除した場合は、30分以内にもアクセスされる可能性がある
        // その点は`series_id_update_at`内でレートリミットを設け対策する
        let should_extend = self.get_last_series_id_extension_time(account_id, series_id)
            .await
            .map(|t| Self::should_extend_series_id_expiration(&t))?;

        if should_extend {
            // 既存のシリーズIDの有効期限を延長する
            self.extend_series_id_expiration(account_id, series_id).await
        } else {
            Ok(())
        }
     */
}

pub enum UpdateRefreshTokenError {
    IssueNewRefreshTokenFailed,
}
