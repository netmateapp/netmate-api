use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{fallible::Fallible, id::account_id::AccountId, region::Region}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{SetRegion, SetRegionError};

pub struct SetRegionImpl {
    db: Arc<Session>,
    update_region: Arc<PreparedStatement>,
}

impl SetRegionImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<SetRegionImpl, InitError<SetRegionImpl>> {
        let update_region = prepare(&db, "UPDATE accounts SET region = ? WHERE id = ?").await?;

        Ok(Self { db, update_region })
    }
}

impl SetRegion for SetRegionImpl {
    async fn set_region(&self, account_id: AccountId, region: Region) -> Fallible<(), SetRegionError> {
        self.db
            .execute_unpaged(&self.update_region, (region, account_id))
            .await
            .map(|_| ())
            .map_err(|e| SetRegionError::SetRegionFailed(e.into()))
    }
}
