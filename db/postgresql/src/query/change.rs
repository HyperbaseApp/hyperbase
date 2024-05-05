use anyhow::Result;
use sqlx::{Executor, Pool, Postgres};

use crate::{db::PostgresDb, model::change::ChangeModel};

const INSERT: &str = "INSERT INTO \"changes\" (\"table\", \"id\", \"state\", \"updated_at\", \"change_id\") VALUES ($1, $2, $3, $4, $5) ON CONFLICT (\"table\", \"id\") DO NOTHING";
const UPSERT: &str = "INSERT INTO \"changes\" (\"table\", \"id\", \"state\", \"updated_at\", \"change_id\") VALUES ($1, $2, $3, $4, $5) ON CONFLICT (\"table\", \"id\") DO UPDATE SET \"state\" = $3, \"updated_at\" = $4, \"change_id\" = $5";
const SELECT_LAST_BY_TABLE: &str = "SELECT \"table\", \"id\", \"state\", \"updated_at\", \"change_id\" FROM \"changes\" WHERE \"table\" = $1 ORDER BY \"updated_at\" DESC, \"change_id\" DESC LIMIT 1";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "[PostgreSQL] Setting up changes table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"changes\" (\"table\" text, \"id\" uuid, \"state\" text, \"updated_at\" timestamptz, \"change_id\" uuid, PRIMARY KEY (\"table\", \"id\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(UPSERT),
        pool.prepare(SELECT_LAST_BY_TABLE),
    )
    .unwrap();
}

impl PostgresDb {
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
}
