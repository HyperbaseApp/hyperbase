use anyhow::Result;
use sqlx::{Executor, Pool, Postgres};

use crate::{db::PostgresDb, model::remote_sync::RemoteSyncModel};

const INSERT: &str = "INSERT INTO \"remotes_sync\" (\"remote_address\", \"remote_id\", \"last_data_sync\") VALUES ($1, $2, $3)";
const SELECT: &str = "SELECT \"remote_address\", \"remote_id\", \"last_data_sync\" FROM \"remotes_sync\" WHERE \"remote_id\" = $1";
const UPDATE: &str = "UPDATE \"remotes_sync\" SET \"remote_id\" = $1, \"last_data_sync\" = $2 WHERE \"remote_address\" = $3";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "[PostgreSQL] Setting up remotes_sync table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"remotes_sync\" (\"remote_address\" text, \"remote_id\" uuid, \"last_data_sync\" timestamptz, PRIMARY KEY (\"remote_address\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT),
        pool.prepare(UPDATE)
    )
    .unwrap();
}

impl PostgresDb {
    pub async fn insert_remote_sync(&self, value: &RemoteSyncModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(value.remote_address())
                .bind(value.remote_id())
                .bind(value.last_data_sync()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_remote_sync(&self, remote_address: &str) -> Result<RemoteSyncModel> {
        Ok(self
            .fetch_one(sqlx::query_as(SELECT).bind(remote_address))
            .await?)
    }

    pub async fn update_remote_sync(&self, value: &RemoteSyncModel) -> Result<()> {
        self.execute(
            sqlx::query(UPDATE)
                .bind(value.remote_id())
                .bind(value.last_data_sync())
                .bind(value.remote_address()),
        )
        .await?;
        Ok(())
    }
}
