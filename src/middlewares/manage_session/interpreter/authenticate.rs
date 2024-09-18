use bb8_redis::redis::cmd;

use crate::{common::{fallible::Fallible, id::account_id::AccountId, session::session_id::SessionId}, helper::redis::conn, middlewares::{manage_session::dsl::authenticate::{AuthenticateSession, AuthenticateSessionError}, session::SessionIdKey}};

use super::ManageSessionImpl;

impl AuthenticateSession for ManageSessionImpl {
    async fn resolve_session_id_to_account_id(&self, session_id: &SessionId) -> Fallible<Option<AccountId>, AuthenticateSessionError> {
        let mut conn = conn(&self.cache, |e| AuthenticateSessionError::ResolveSessionIdFailed(e.into())).await?;
        
        cmd("GET")
            .arg(SessionIdKey::new(session_id))
            .query_async::<Option<AccountId>>(&mut *conn)
            .await
            .map_err(|e| AuthenticateSessionError::ResolveSessionIdFailed(e.into()))
    }
}