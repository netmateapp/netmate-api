use thiserror::Error;

use crate::common::{fallible::Fallible, handle::{id::HandleId, share_count::HandleShareCount}, id::account_id::AccountId};

pub(crate) trait CountHandlesShare {
    async fn count_handles_share(&self, account_id: AccountId) -> Fallible<Vec<(HandleId, HandleShareCount)>, CountHandlesShareError>;
}

#[derive(Debug, Error)]
pub enum CountHandlesShareError {
    #[error("アカウントの各名義の共有数のカウントに失敗しました")]
    CountHandlesShareFailed(#[source] anyhow::Error),
}