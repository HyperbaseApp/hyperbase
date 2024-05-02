use anyhow::Result;
use sqlx::{Executor, Pool, Sqlite};

use crate::{db::SqliteDb, model::remote_sync::RemoteSyncModel};

const INSERT: &str = "INSERT INTO \"remotes_sync\" (\"remote_address\", \"remote_id\", \"last_data_sync\") VALUES (?, ?, ?)";
const SELECT: &str = "SELECT \"remote_address\", \"remote_id\", \"last_data_sync\" FROM \"remotes_sync\" WHERE \"remote_id\" = ?";
const UPDATE: &str = "UPDATE \"remotes_sync\" SET \"remote_id\" = ?, \"last_data_sync\" = ? WHERE \"remote_address\" = ?";

pub async fn init(pool: &Pool<Sqlite>) {
    hb_log::info(Some("🔧"), "[SQLite] Setting up remotes_sync table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"remotes_sync\" (\"remote_address\" text, \"remote_id\" blob, \"last_data_sync\" timestamp, PRIMARY KEY (\"remote_address\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT),
        pool.prepare(UPDATE)
    )
    .unwrap();
}

impl SqliteDb {
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
