use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Executor, Pool, Postgres};

use crate::{db::PostgresDb, model::change::ChangeModel};

const INSERT: &str = "INSERT INTO \"changes\" (\"table\", \"id\", \"state\", \"updated_at\") VALUES ($1, $2, $3, $4)";
const SELECT_MANY: &str = "SELECT \"table\", \"id\", \"state\", \"updated_at\" FROM \"changes\" ORDER BY \"updated_at\" DESC";
const SELECT_MANY_FROM_TIME: &str = "SELECT \"table\", \"id\", \"state\", \"updated_at\" FROM \"changes\" WHERE \"updated_at\" >= $1 ORDER BY \"updated_at\" DESC";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "[PostgreSQL] Setting up changes table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"changes\" (\"table\" text, \"id\" uuid, \"state\" text, \"updated_at\" timestamptz, PRIMARY KEY (\"table\", \"id\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT_MANY),
        pool.prepare(SELECT_MANY_FROM_TIME),
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
                .bind(value.updated_at()),
        )
        .await?;
        Ok(())
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
}
