use bb8_redis::redis::cmd;

use crate::{common::{fallible::Fallible, id::account_id::AccountId, session::{refresh_pair_expiration::RefreshPairExpirationSeconds, refresh_token::RefreshToken, session_series::SessionSeries}}, helper::redis::{Connection, TypedCommand, EX_OPTION, SET_COMMAND}, middlewares::{manage_session::dsl::update_refresh_token::{UpdateRefreshToken, UpdateRefreshTokenError}, session::{RefreshPairKey, RefreshPairValue}}};

use super::ManageSessionImpl;

impl UpdateRefreshToken for ManageSessionImpl {
    async fn assign_new_refresh_token_with_expiration(&self, new_refresh_token: &RefreshToken, session_series: &SessionSeries, session_account_id: AccountId, expiration: RefreshPairExpirationSeconds) -> Fallible<(), UpdateRefreshTokenError> {
        let key = RefreshPairKey(session_series);
        let value = RefreshPairValue(new_refresh_token, session_account_id);

        SetNewRefreshTokenCommand.run(&self.cache, (key, value, expiration))
            .await
            .map_err(UpdateRefreshTokenError::AssignNewRefreshTokenFailed)
    }
}

struct SetNewRefreshTokenCommand;

impl<'a, 'b> TypedCommand<(RefreshPairKey<'a>, RefreshPairValue<'b>, RefreshPairExpirationSeconds), ()> for SetNewRefreshTokenCommand {
    async fn execute(&self, mut conn: Connection<'_>, (key, value, expiration): (RefreshPairKey<'a>, RefreshPairValue<'b>, RefreshPairExpirationSeconds)) -> anyhow::Result<()> {
        cmd(SET_COMMAND)
            .arg(key)
            .arg(value)
            .arg(EX_OPTION)
            .arg(expiration)
            .exec_async(&mut *conn)
            .await
            .map_err(Into::into)
    }
}