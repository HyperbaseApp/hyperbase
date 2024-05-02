use anyhow::Result;
use scylla::{
    frame::value::CqlTimestamp,
    transport::{query_result::FirstRowTypedError, session::TypedRowIter},
    CachingSession,
};
use uuid::Uuid;

use crate::{db::ScyllaDb, model::change::ChangeModel};

const INSERT: &str = "INSERT INTO \"hyperbase\".\"changes\" (\"table\", \"id\", \"state\", \"updated_at\", \"change_id\") VALUES (?, ?, ?, ?, ?)";
const SELECT_LAST: &str = "SELECT \"table\", \"id\", \"state\", \"updated_at\", \"change_id\" FROM \"hyperbase\".\"changes\" ORDER BY \"updated_at\" DESC LIMIT 1";
const SELECT_MANY: &str = "SELECT \"table\", \"id\", \"state\", \"updated_at\", \"change_id\" FROM \"hyperbase\".\"changes\" ORDER BY \"updated_at\" DESC";
const SELECT_MANY_FROM_TIME: &str = "SELECT \"table\", \"id\", \"state\", \"updated_at\", \"change_id\" FROM \"hyperbase\".\"changes\" WHERE \"updated_at\" >= ? ORDER BY \"updated_at\" DESC";
const DELETE_BY_TABLE_AND_ID: &str = "DELETE FROM \"hyperbase\".\"changes\" WHERE \"table\" = ? AND \"id\" = ?";

pub async fn init(cached_session: &CachingSession) {
    hb_log::info(Some("🔧"), "[ScyllaDB] Setting up changes table");

    cached_session
        .get_session()
        .query("CREATE TABLE IF NOT EXISTS ON \"hyperbase\".\"changes\" (\"table\" text, \"id\" uuid, \"state\" text, \"updated_at\" timestamp, \"change_id\" uuid, PRIMARY KEY ((\"table\", \"id\"), \"updated_at\"))", &[])
        .await
        .unwrap();

    cached_session
        .add_prepared_statement(&INSERT.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT_LAST.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT_MANY.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT_MANY_FROM_TIME.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&DELETE_BY_TABLE_AND_ID.into())
        .await
        .unwrap();
}

impl ScyllaDb {
    pub async fn insert_change(&self, value: &ChangeModel) -> Result<()> {
        self.execute(INSERT, value).await?;
        Ok(())
    }

    pub async fn select_last_change(&self) -> Result<Option<ChangeModel>> {
        let data = self.execute(SELECT_LAST, &[]).await?.first_row_typed();
        match data {
            Ok(data) => Ok(Some(data)),
            Err(err) => {
                if matches!(err, FirstRowTypedError::RowsEmpty) {
                    Ok(None)
                } else {
                    Err(err.into())
                }
            }
        }
    }

    pub async fn select_many_changes(&self) -> Result<TypedRowIter<ChangeModel>> {
        Ok(self.execute(SELECT_MANY, &[]).await?.rows_typed()?)
    }

    pub async fn select_many_changes_from_time(
        &self,
        time: &CqlTimestamp,
    ) -> Result<TypedRowIter<ChangeModel>> {
        Ok(self
            .execute(SELECT_MANY_FROM_TIME, [time].as_ref())
            .await?
            .rows_typed()?)
    }

    pub async fn delete_change(&self, table: &str, id: &Uuid) -> Result<()> {
        self.execute(DELETE_BY_TABLE_AND_ID, &(table, id)).await?;
        Ok(())
    }
}
