use bb8_redis::redis::cmd;
use uuid::Uuid;

use crate::{common::{fallible::Fallible, id::{uuid7::Uuid7, AccountId}, session::value::SessionId}, helper::redis::conn, middlewares::manage_session::{dsl::authenticate::{AuthenticateSession, AuthenticateSessionError}, interpreter::SESSION_ID_NAMESPACE}};

use super::ManageSessionImpl;


impl AuthenticateSession for ManageSessionImpl {
    async fn resolve_session_id_to_account_id(&self, session_id: &SessionId) -> Fallible<Option<AccountId>, AuthenticateSessionError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> AuthenticateSessionError {
            AuthenticateSessionError::ResolveSessionIdFailed(e.into())
        }
        
        let mut conn = conn(&self.cache, handle_error)
            .await?;

        let key = format!("{}:{}", SESSION_ID_NAMESPACE, session_id.to_string());

        cmd("GET")
            .arg(key)
            .query_async::<Option<Uuid>>(&mut *conn)
            .await
            .map_err(handle_error)?
            .map(|uuid| Uuid7::try_from(uuid))
            .transpose()
            .map_or_else(|e| Err(handle_error(e)), |o| Ok(o.map(AccountId::new)))
    }
}