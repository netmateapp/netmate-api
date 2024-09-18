use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{email::address::Email, fallible::Fallible, id::account_id::AccountId, password::PasswordHash}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{SignIn, SignInError};

pub struct SignInImpl {
    db: Arc<Session>,
    select_password_hash_and_account_id: Arc<PreparedStatement>,
}

impl SignInImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<Self, InitError<Self>> {
        let select_password_hash_and_account_id = prepare(&db, "SELECT password_hash, id FROM accounts WHERE email = ? LIMIT 1").await?;

        Ok(Self { db, select_password_hash_and_account_id })
    }
}

impl SignIn for SignInImpl {
    async fn fetch_password_hash_and_account_id(&self, email: &Email) -> Fallible<Option<(PasswordHash, AccountId)>, SignInError> {
        self.db
            .execute_unpaged(&self.select_password_hash_and_account_id, (email, ))
            .await
            .map_err(|e| SignInError::FetchPasswordHashAndAccountIdFailed(e.into()))?
            .maybe_first_row_typed()
            .map_err(|e| SignInError::FetchPasswordHashAndAccountIdFailed(e.into()))
    }
}