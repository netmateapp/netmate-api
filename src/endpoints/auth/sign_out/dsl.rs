use thiserror::Error;

use crate::common::{fallible::Fallible, id::account_id::AccountId, session::session_series::SessionSeries};

pub(crate) trait SignOut {
    async fn sign_out(&self, account_id: AccountId, session_series: &SessionSeries) -> Fallible<(), SignOutError>;
}

#[derive(Debug, Error)]
pub enum SignOutError {
    #[error("ログアウトに失敗しました")]
    SignOutFailed(#[source] anyhow::Error),
}