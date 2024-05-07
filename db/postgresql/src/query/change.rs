use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Executor, Pool, Postgres};
use uuid::Uuid;

use crate::{db::PostgresDb, model::change::ChangeModel};

const INSERT_OR_IGNORE: &str = "INSERT INTO \"changes\" (\"table\", \"id\", \"state\", \"updated_at\", \"change_id\") VALUES ($1, $2, $3, $4, $5) ON CONFLICT (\"table\", \"id\") DO NOTHING";
const UPSERT: &str = "INSERT INTO \"changes\" (\"table\", \"id\", \"state\", \"updated_at\", \"change_id\") VALUES ($1, $2, $3, $4, $5) ON CONFLICT (\"table\", \"id\") DO UPDATE SET \"state\" = $3, \"updated_at\" = $4, \"change_id\" = $5";
const SELECT_LAST_BY_TABLE: &str = "SELECT \"table\", \"id\", \"state\", \"updated_at\", \"change_id\" FROM \"changes\" WHERE \"table\" = $1 ORDER BY \"updated_at\" DESC, \"change_id\" DESC LIMIT 1";
const SELECT_MANY_BY_CHANGE_IDS_ASC: &str = "SELECT \"table\", \"id\", \"state\", \"updated_at\", \"change_id\" FROM \"changes\" WHERE \"change_id\" IN ($1) ORDER BY \"updated_at\" ASC, \"change_id\" ASC";
const SELECT_MANY_FROM_UPDATED_AT_AND_AFTER_CHANGE_ID_WITH_LIMIT_ASC: &str = "SELECT \"table\", \"id\", \"state\", \"updated_at\", \"change_id\" FROM \"changes\" WHERE \"updated_at\" > $1 OR (\"updated_at\" = $1 AND \"change_id\" > $2) ORDER BY \"updated_at\" ASC, \"change_id\" ASC LIMIT $3";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "[PostgreSQL] Setting up changes table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"changes\" (\"table\" text, \"id\" uuid, \"state\" text, \"updated_at\" timestamptz, \"change_id\" uuid, PRIMARY KEY (\"table\", \"id\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT_OR_IGNORE),
        pool.prepare(UPSERT),
        pool.prepare(SELECT_LAST_BY_TABLE),
        pool.prepare(SELECT_MANY_BY_CHANGE_IDS_ASC),
        pool.prepare(SELECT_MANY_FROM_UPDATED_AT_AND_AFTER_CHANGE_ID_WITH_LIMIT_ASC),
    )
    .unwrap();
}

impl PostgresDb {
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
        Ok(self
            .fetch_all(sqlx::query_as(SELECT_MANY_BY_CHANGE_IDS_ASC).bind(change_ids))
            .await?)
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
                    .bind(change_id)
                    .bind(limit),
            )
            .await?)
    }
}
