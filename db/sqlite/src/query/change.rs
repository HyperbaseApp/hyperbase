use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Executor, Pool, Sqlite};
use uuid::Uuid;

use crate::{db::SqliteDb, model::change::ChangeModel};

const INSERT_OR_IGNORE: &str = "INSERT INTO \"changes\" (\"table\", \"id\", \"state\", \"updated_at\", \"change_id\") VALUES (?, ?, ?, ?, ?) ON CONFLICT (\"table\", \"id\") DO NOTHING";
const UPSERT: &str = "INSERT INTO \"changes\" (\"table\", \"id\", \"state\", \"updated_at\", \"change_id\") VALUES (?, ?, ?, ?, ?) ON CONFLICT (\"table\", \"id\") DO UPDATE SET \"state\" = ?, \"updated_at\" = ?, \"change_id\" = ?";
const SELECT_LAST_BY_TABLE: &str = "SELECT \"table\", \"id\", \"state\", \"updated_at\", \"change_id\" FROM \"changes\" WHERE \"table\" = ? ORDER BY \"updated_at\" DESC, \"change_id\" DESC LIMIT 1";
const SELECT_MANY_BY_CHANGE_IDS_ASC: &str = "SELECT \"table\", \"id\", \"state\", \"updated_at\", \"change_id\" FROM \"changes\" WHERE \"change_id\" IN (?) ORDER BY \"updated_at\" ASC, \"change_id\" ASC";
const SELECT_MANY_FROM_UPDATED_AT_AND_AFTER_CHANGE_ID_WITH_LIMIT_ASC: &str = "SELECT \"table\", \"id\", \"state\", \"updated_at\", \"change_id\" FROM \"changes\" WHERE \"updated_at\" > ? OR (\"updated_at\" = ? AND \"change_id\" > ?) ORDER BY \"updated_at\" ASC, \"change_id\" ASC LIMIT ?";

pub async fn init(pool: &Pool<Sqlite>) {
    hb_log::info(Some("🔧"), "[SQLite] Setting up changes table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"changes\" (\"table\" text, \"id\" blob, \"state\" text, \"updated_at\" timestamp, \"change_id\" blob, PRIMARY KEY (\"table\", \"id\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT_OR_IGNORE),
        pool.prepare(UPSERT),
        pool.prepare(SELECT_LAST_BY_TABLE),
        pool.prepare(SELECT_MANY_BY_CHANGE_IDS_ASC),
        pool.prepare(SELECT_MANY_FROM_UPDATED_AT_AND_AFTER_CHANGE_ID_WITH_LIMIT_ASC),
    )
    .unwrap();
}

impl SqliteDb {
    pub async fn insert_or_ignore_change(&self, value: &ChangeModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT_OR_IGNORE)
                .bind(value.table())
                .bind(value.id())
                .bind(value.state())
                .bind(value.updated_at())
                .bind(value.change_id()),
        )
        .await?;
        Ok(())
    }

    pub async fn upsert_change(&self, value: &ChangeModel) -> Result<()> {
        self.execute(
            sqlx::query(UPSERT)
                .bind(value.table())
                .bind(value.id())
                .bind(value.state())
                .bind(value.updated_at())
                .bind(value.change_id())
                .bind(value.state())
                .bind(value.updated_at())
                .bind(value.change_id()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_last_change_by_table(&self, table: &str) -> Result<Option<ChangeModel>> {
        let data = self
            .fetch_one(sqlx::query_as(SELECT_LAST_BY_TABLE).bind(table))
            .await;
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

    pub async fn select_many_changes_by_change_ids_asc(
        &self,
        change_ids: &Vec<Uuid>,
    ) -> Result<Vec<ChangeModel>> {
        let mut parameter_binding = Vec::with_capacity(change_ids.len());
        for _ in 0..change_ids.len() {
            parameter_binding.push("?");
        }
        let parameter_binding = format!("({})", parameter_binding.join(", "));

        let query = SELECT_MANY_BY_CHANGE_IDS_ASC.replacen("(?)", &parameter_binding, 1);

        let mut query = sqlx::query_as(&query);
        for change_id in change_ids {
            query = query.bind(change_id);
        }

        Ok(self.fetch_all(query).await?)
    }

    pub async fn select_many_changes_from_updated_at_and_after_change_id_with_limit_asc(
        &self,
        updated_at: &DateTime<Utc>,
        change_id: &Uuid,
        limit: &i32,
    ) -> Result<Vec<ChangeModel>> {
        Ok(self
            .fetch_all(
                sqlx::query_as(SELECT_MANY_FROM_UPDATED_AT_AND_AFTER_CHANGE_ID_WITH_LIMIT_ASC)
                    .bind(updated_at)
                    .bind(updated_at)
                    .bind(change_id)
                    .bind(limit),
            )
            .await?)
    }
}
