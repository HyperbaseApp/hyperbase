use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Executor, Pool, Sqlite};

use crate::{db::SqliteDb, model::change::ChangeModel};

const INSERT: &str = "INSERT INTO \"changes\" (\"table\", \"id\", \"state\", \"updated_at\", \"change_id\") VALUES (?, ?, ?, ?, ?)";
const SELECT_LAST: &str = "SELECT \"table\", \"id\", \"state\", \"updated_at\", \"change_id\" FROM \"changes\" ORDER BY \"updated_at\" DESC LIMIT 1";
const SELECT_MANY: &str = "SELECT \"table\", \"id\", \"state\", \"updated_at\", \"change_id\" FROM \"changes\" ORDER BY \"updated_at\" DESC";
const SELECT_MANY_FROM_TIME: &str = "SELECT \"table\", \"id\", \"state\", \"updated_at\", \"change_id\" FROM \"changes\" WHERE \"updated_at\" >= ? ORDER BY \"updated_at\" DESC";
const UPDATE: &str = "UPDATE \"changes\" SET \"updated_at\" = ?, \"state\" = ?, \"change_id\" = ? WHERE \"table\" = ? AND \"id\" = ?";

pub async fn init(pool: &Pool<Sqlite>) {
    hb_log::info(Some("🔧"), "[SQLite] Setting up changes table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"changes\" (\"table\" text, \"id\" blob, \"state\" text, \"updated_at\" timestamp, \"change_id\" blob, PRIMARY KEY (\"table\", \"id\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT_MANY),
        pool.prepare(SELECT_MANY_FROM_TIME),
        pool.prepare(UPDATE),
    )
    .unwrap();
}

impl SqliteDb {
    pub async fn insert_change(&self, value: &ChangeModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(value.table())
                .bind(value.id())
                .bind(value.state())
                .bind(value.updated_at())
                .bind(value.change_id()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_last_change(&self) -> Result<Option<ChangeModel>> {
        let data = self.fetch_one(sqlx::query_as(SELECT_LAST)).await;
        match data {
            Ok(data) => Ok(Some(data)),
            Err(err) => {
                if matches!(err, sqlx::Error::RowNotFound) {
                    Ok(None)
                } else {
                    Err(err.into())
                }
            }
        }
    }

    pub async fn select_many_changes(&self) -> Result<Vec<ChangeModel>> {
        Ok(self.fetch_all(sqlx::query_as(SELECT_MANY)).await?)
    }

    pub async fn select_many_changes_from_time(
        &self,
        time: &DateTime<Utc>,
    ) -> Result<Vec<ChangeModel>> {
        Ok(self
            .fetch_all(sqlx::query_as(SELECT_MANY_FROM_TIME).bind(time))
            .await?)
    }

    pub async fn update_change(&self, value: &ChangeModel) -> Result<()> {
        self.execute(
            sqlx::query(UPDATE)
                .bind(value.updated_at())
                .bind(value.state())
                .bind(value.change_id())
                .bind(value.table())
                .bind(value.id()),
        )
        .await?;
        Ok(())
    }
}
