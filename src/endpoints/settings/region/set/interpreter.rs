use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, FromRow, Session};

use crate::{common::{fallible::Fallible, id::account_id::AccountId, region::Region}, helper::{error::InitError, scylla::{Statement, TypedStatement, Unit}}};

use super::dsl::{SetRegion, SetRegionError};

pub struct SetRegionImpl {
    db: Arc<Session>,
    update_region: Arc<UpdateRegion>,
}

impl SetRegionImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<SetRegionImpl, InitError<SetRegionImpl>> {
        let update_region = UPDATE_REGION.prepared(&db, UpdateRegion)
            .await
            .map_err(|e| InitError::new(e.into()))?;

        Ok(Self { db, update_region })
    }
}

impl SetRegion for SetRegionImpl {
    async fn set_region(&self, account_id: AccountId, region: Region) -> Fallible<(), SetRegionError> {
        self.update_region
            .execute(&self.db, (region, account_id))
            .await
            .map_err(SetRegionError::SetRegionFailed)
    }
}

const UPDATE_REGION: Statement<UpdateRegion>
    = Statement::of("UPDATE accounts SET region = ? WHERE id = ?");

struct UpdateRegion(PreparedStatement);

impl TypedStatement<(Region, AccountId), Unit> for UpdateRegion {
    type Result<U> = U where U: FromRow;

    async fn query(&self, session: &Arc<Session>, values: (Region, AccountId)) -> anyhow::Result<Unit> {
        session.execute_unpaged(&self.0, values)
            .await
            .map(|_| Unit)
            .map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::helper::scylla::check_cql_statement_type;

    use super::UPDATE_REGION;

    #[test]
    fn check_update_region_type() {
        check_cql_statement_type(UPDATE_REGION);
    }
}