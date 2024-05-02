use anyhow::Result;
use scylla::CachingSession;

use crate::{db::ScyllaDb, model::remote_sync::RemoteSyncModel};

const INSERT: &str = "INSERT INTO \"hyperbase\".\"remotes_sync\" (\"remote_address\", \"remote_id\", \"last_data_sync\") VALUES (?, ?, ?)";
const SELECT: &str = "SELECT \"remote_address\", \"remote_id\", \"last_data_sync\" FROM \"hyperbase\".\"remotes_sync\" WHERE \"remote_id\" = ?";
const UPDATE: &str = "UPDATE \"hyperbase\".\"remotes_sync\" SET \"remote_id\" = ?, \"last_data_sync\" = ? WHERE \"remote_address\" = ?";

pub async fn init(cached_session: &CachingSession) {
    hb_log::info(Some("🔧"), "[ScyllaDB] Setting up remotes_sync table");

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"remotes_sync\" (\"remote_address\" text, \"remote_id\" uuid, \"last_data_sync\" timestamp, PRIMARY KEY (\"remote_address\"))", &[]).await.unwrap();

    cached_session
        .add_prepared_statement(&INSERT.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&UPDATE.into())
        .await
        .unwrap();
}

impl ScyllaDb {
    pub async fn insert_remote_sync(&self, value: &RemoteSyncModel) -> Result<()> {
        self.execute(INSERT, value).await?;
        Ok(())
    }

    pub async fn select_remote_sync(&self, remote_address: &str) -> Result<RemoteSyncModel> {
        Ok(self
            .execute(SELECT, [remote_address].as_ref())
            .await?
            .first_row_typed()?)
    }

    pub async fn update_remote_sync(&self, value: &RemoteSyncModel) -> Result<()> {
        self.execute(
            UPDATE,
            &(
                value.remote_id(),
                value.last_data_sync(),
                value.remote_address(),
            ),
        )
        .await?;
        Ok(())
    }
}
