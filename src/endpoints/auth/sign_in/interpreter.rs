use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, FromRow, Session};

use crate::{common::{email::address::Email, fallible::Fallible, id::account_id::AccountId, password::PasswordHash}, helper::{error::InitError, scylla::{Statement, TypedStatement}}};

use super::dsl::{SignIn, SignInError};

pub struct SignInImpl {
    db: Arc<Session>,
    select_password_hash_and_account_id: Arc<SelectPasswordHashAndAccountId>,
}

impl SignInImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<Self, InitError<Self>> {
        let select_password_hash_and_account_id = SELECT_PASSWORD_HASH_AND_ACCOUNT_ID.prepared(&db, SelectPasswordHashAndAccountId)
            .await
            .map_err(|e| InitError::new(e.into()))?;

        Ok(Self { db, select_password_hash_and_account_id })
    }
}

impl SignIn for SignInImpl {
    async fn fetch_password_hash_and_account_id(&self, email: &Email) -> Fallible<Option<(PasswordHash, AccountId)>, SignInError> {
        self.select_password_hash_and_account_id
            .query(&self.db, (email, ))
            .await
            .map_err(|e| SignInError::FetchPasswordHashAndAccountIdFailed(e.into()))
    }
}

const SELECT_PASSWORD_HASH_AND_ACCOUNT_ID: Statement<SelectPasswordHashAndAccountId>
    = Statement::of("SELECT password_hash, id FROM accounts WHERE email = ? LIMIT 1");

struct SelectPasswordHashAndAccountId(PreparedStatement);

impl<'a> TypedStatement<(&'a Email, ), (PasswordHash, AccountId)> for SelectPasswordHashAndAccountId {
    type Result<U> = Option<U> where U: FromRow;

    async fn query(&self, session: &Arc<Session>, values: (&'a Email, )) -> anyhow::Result<Self::Result<(PasswordHash, AccountId)>> {
        session.execute_unpaged(&self.0, values)
            .await
            .map_err(anyhow::Error::from)?
            .maybe_first_row_typed()
            .map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::helper::scylla::check_cql_query_type;

    use super::SELECT_PASSWORD_HASH_AND_ACCOUNT_ID;

    #[test]
    fn check_select_password_hash_and_account_id_type() {
        check_cql_query_type(SELECT_PASSWORD_HASH_AND_ACCOUNT_ID);
    }
}