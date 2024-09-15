use thiserror::Error;

use crate::common::{email::address::Email, fallible::Fallible, id::account_id::{AccountId, EMPTY_ACCOUNT_ID}, password::{Password, PasswordHash, EMPTY_PASSWORD_HASH}};

pub(crate) trait SignIn {
    async fn sign_in(&self, email: &Email, password: &Password) -> Fallible<Option<AccountId>, SignInError> {
        // 時間差攻撃を防ぐためメールアドレスが存在しない場合もパスワードの検証を行う
        let (password_hash, account_id) = self.fetch_password_hash_and_account_id(email)
            .await?
            .unwrap_or_else(|| (EMPTY_PASSWORD_HASH.clone(), EMPTY_ACCOUNT_ID));

        if password_hash.verify(password) {
            Ok(Some(account_id))
        } else {
            Ok(None)
        }
    }

    async fn fetch_password_hash_and_account_id(&self, email: &Email) -> Fallible<Option<(PasswordHash, AccountId)>, SignInError>;
}

#[derive(Debug, Error)]
pub enum SignInError {
    #[error("パスワードハッシュとアカウントIDの取得に失敗しました")]
    FetchPasswordHashAndAccountIdFailed(#[source] anyhow::Error),
}