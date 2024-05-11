use anyhow::Result;
use sqlx::{Executor, MySql, Pool};
use uuid::Uuid;

use crate::{db::MysqlDb, model::remote_sync::RemoteSyncModel};

const INSERT_OR_IGNORE: &str = "INSERT INTO `remotes_sync` (`remote_id`, `remote_address`, `last_data_sync`, `last_change_id`) VALUES (?, ?, ?, ?) ON DUPLICATE KEY UPDATE `remote_id` = ?";
const UPSERT: &str = "INSERT INTO `remotes_sync` (`remote_id`, `remote_address`, `last_data_sync`, `last_change_id`) VALUES (?, ?, ?, ?) ON DUPLICATE KEY UPDATE `remote_address` = ?, `last_data_sync` = ?, `last_change_id` = ?";
const SELECT: &str = "SELECT `remote_id`, `remote_address`, `last_data_sync`, `last_change_id` FROM `remotes_sync` WHERE `remote_id` = ?";
const SELECT_MANY_BY_REMOTE_ADDRESS: &str = "SELECT `remote_id`, `remote_address`, `last_data_sync`, `last_change_id` FROM `remotes_sync` WHERE `remote_address` = ?";

pub async fn init(pool: &Pool<MySql>) {
    hb_log::info(Some("🔧"), "[MySQL] Setting up remotes_sync table");

    pool.execute("CREATE TABLE IF NOT EXISTS `remotes_sync` (`remote_id` binary(16), `remote_address` text, `last_data_sync` timestamp(6), `last_change_id` binary(16), PRIMARY KEY (`remote_id`))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT_OR_IGNORE),
        pool.prepare(UPSERT),
        pool.prepare(SELECT),
        pool.prepare(SELECT_MANY_BY_REMOTE_ADDRESS),
    )
    .unwrap();
}

impl MysqlDb {
    pub async fn insert_or_ignore_remote_sync(&self, value: &RemoteSyncModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT_OR_IGNORE)
                .bind(value.remote_id())
                .bind(value.remote_address())
                .bind(value.last_data_sync())
                .bind(value.last_change_id())
                .bind(value.remote_id()),
        )
        .await?;
        Ok(())
    }

    pub async fn upsert_remote_sync(&self, value: &RemoteSyncModel) -> Result<()> {
        self.execute(
            sqlx::query(UPSERT)
                .bind(value.remote_id())
                .bind(value.remote_address())
                .bind(value.last_data_sync())
                .bind(value.last_change_id())
                .bind(value.remote_address())
                .bind(value.last_data_sync())
                .bind(value.last_change_id()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_remote_sync(&self, remote_id: &Uuid) -> Result<RemoteSyncModel> {
        Ok(self
            .fetch_one(sqlx::query_as(SELECT).bind(remote_id))
            .await?)
    }

    pub async fn select_many_remotes_sync_by_address(
        &self,
        remote_address: &str,
    ) -> Result<Vec<RemoteSyncModel>> {
        Ok(self
            .fetch_all(sqlx::query_as(SELECT_MANY_BY_REMOTE_ADDRESS).bind(remote_address))
            .await?)
    }
}
