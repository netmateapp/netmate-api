use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, FromRow, Session};

use crate::{common::{fallible::Fallible, handle::{id::HandleId, name::HandleName, share_count::HandleShareCount}, id::account_id::AccountId}, helper::{error::InitError, scylla::{Statement, TypedStatement}}};

use super::dsl::{GetHandles, GetHandlesError};

pub struct GetHandlesImpl {
    db: Arc<Session>,
    select_handles: Arc<SelectHandles>,
    select_handle_share_counts: Arc<SelectHandleShareCounts>,
}

impl GetHandlesImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<Self, InitError<Self>> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> InitError<GetHandlesImpl> {
            InitError::new(e.into())
        }

        let select_handles = SELECT_HANDLES.prepared(&db, SelectHandles)
            .await
            .map_err(handle_error)?;

        let select_handle_share_counts = SELECT_HANDLE_SHARE_COUNTS.prepared(&db, SelectHandleShareCounts)
            .await
            .map_err(handle_error)?;

        Ok(Self { db, select_handles, select_handle_share_counts })
    }
}

impl GetHandles for GetHandlesImpl {
    async fn get_handles(&self, account_id: AccountId) -> Fallible<Vec<(HandleId, HandleName, HandleShareCount)>, GetHandlesError> {
        let handles = self.select_handles.query(&self.db, (account_id, ))
            .await
            .map_err(GetHandlesError::GetHandlesFailed)?;

        let handle_share_counts = self.select_handle_share_counts.query(&self.db, (account_id, ))
            .await
            .map_err(GetHandlesError::GetHandlesFailed)?;

        let handles = handles.into_iter()
            .zip(handle_share_counts.into_iter())
            .map(|((handle_id, handle_name), (handle_share_count, ))| (handle_id, handle_name, handle_share_count))
            .collect();

        Ok(handles)
    }
}

const SELECT_HANDLES: Statement<SelectHandles>
    = Statement::of("SELECT handle_id, handle_name FROM account_handles WHERE account_id = ?");

struct SelectHandles(PreparedStatement);

impl TypedStatement<(AccountId, ), (HandleId, HandleName)> for SelectHandles {
    type Result<U> = Vec<U> where U: FromRow;

    async fn query(&self, session: &Arc<Session>, values: (AccountId, )) -> anyhow::Result<Self::Result<(HandleId, HandleName)>> {
        session.execute_unpaged(&self.0, values)
            .await
            .map_err(anyhow::Error::from)?
            .rows_typed()
            .map(|rows| {
                rows.flatten()
                    .collect::<Vec<(HandleId, HandleName)>>()
            })
            .map_err(anyhow::Error::from)
    }
}

const SELECT_HANDLE_SHARE_COUNTS: Statement<SelectHandleShareCounts>
    = Statement::of("SELECT share_count FROM handle_share_counts WHERE account_id = ?");

struct SelectHandleShareCounts(PreparedStatement);

impl TypedStatement<(AccountId, ), (HandleShareCount, )> for SelectHandleShareCounts {
    type Result<U> = Vec<U> where U: FromRow;

    async fn query(&self, session: &Arc<Session>, values: (AccountId, )) -> anyhow::Result<Self::Result<(HandleShareCount, )>> {
        session.execute_unpaged(&self.0, values)
            .await
            .map_err(anyhow::Error::from)?
            .rows_typed()
            .map(|rows| {
                rows.flatten()
                    .collect::<Vec<(HandleShareCount, )>>()
            })
            .map_err(anyhow::Error::from)
    }
}