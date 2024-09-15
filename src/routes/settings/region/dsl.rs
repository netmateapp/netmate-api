use thiserror::Error;

use crate::common::{fallible::Fallible, id::account_id::AccountId, region::Region};

pub(crate) trait SetRegion {
    async fn set_region(&self, account_id: AccountId, region: Region) -> Fallible<(), SetRegionError>;
}

#[derive(Debug, Error)]
pub enum SetRegionError {
    #[error("地域の設定に失敗しました")]
    SetRegionFailed(#[source] anyhow::Error),
}