use std::{net::SocketAddr, str::FromStr};

use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use hb_db_mysql::model::remote_sync::RemoteSyncModel as RemoteSyncMysqlModel;
use hb_db_postgresql::model::remote_sync::RemoteSyncModel as RemoteSyncPostgresModel;
use hb_db_sqlite::model::remote_sync::RemoteSyncModel as RemoteSyncSqliteModel;
use uuid::Uuid;

use crate::Db;

#[derive(Clone, Copy)]
pub struct RemoteSyncDao {
    remote_id: Uuid,
    remote_address: SocketAddr,
    last_data_sync: DateTime<Utc>,
    last_change_id: Uuid,
}

impl RemoteSyncDao {
    pub fn new(
        remote_id: &Uuid,
        remote_address: &SocketAddr,
        last_data_sync: &DateTime<Utc>,
        last_change_id: &Uuid,
    ) -> Self {
        Self {
            remote_id: *remote_id,
            remote_address: *remote_address,
            last_data_sync: *last_data_sync,
            last_change_id: *last_change_id,
        }
    }

    pub fn remote_id(&self) -> &Uuid {
        &self.remote_id
    }

    pub fn remote_address(&self) -> &SocketAddr {
        &self.remote_address
    }

    pub fn last_data_sync(&self) -> &DateTime<Utc> {
        &self.last_data_sync
    }

    pub fn last_change_id(&self) -> &Uuid {
        &self.last_change_id
    }

    pub fn set_remote_id(&mut self, remote_id: &Uuid) {
        self.remote_id = *remote_id;
    }

    pub fn set_last_data_sync(&mut self, last_data_sync: &DateTime<Utc>) {
        self.last_data_sync = *last_data_sync;
    }

    pub async fn db_insert_or_ignore(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(_) => Err(Error::msg("Unimplemented")),
            Db::PostgresqlDb(db) => {
                db.insert_or_ignore_remote_sync(&self.to_postgresdb_model())
                    .await
            }
            Db::MysqlDb(db) => {
                db.insert_or_ignore_remote_sync(&self.to_mysqldb_model())
                    .await
            }
            Db::SqliteDb(db) => {
                db.insert_or_ignore_remote_sync(&self.to_sqlitedb_model())
                    .await
            }
        }
    }

    pub async fn db_upsert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(_) => Err(Error::msg("Unimplemented")),
            Db::PostgresqlDb(db) => db.upsert_remote_sync(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.upsert_remote_sync(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.upsert_remote_sync(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_select(db: &Db, remote_id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(_) => Err(Error::msg("Unimplemented")),
            Db::PostgresqlDb(db) => {
                Self::from_postgresdb_model(&db.select_remote_sync(remote_id).await?)
            }
            Db::MysqlDb(db) => Self::from_mysqldb_model(&db.select_remote_sync(remote_id).await?),
            Db::SqliteDb(db) => Self::from_sqlitedb_model(&db.select_remote_sync(remote_id).await?),
        }
    }

    pub async fn db_select_many_by_address(
        db: &Db,
        remote_address: &SocketAddr,
    ) -> Result<Vec<Self>> {
        let remote_address = remote_address.to_string();
        match db {
            Db::ScyllaDb(_) => Err(Error::msg("Unimplemented")),
            Db::PostgresqlDb(db) => {
                let remotes = db
                    .select_many_remotes_sync_by_address(&remote_address)
                    .await?;
                let mut remotes_data = Vec::with_capacity(remotes.len());
                for remote in &remotes {
                    remotes_data.push(Self::from_postgresdb_model(remote)?);
                }
                Ok(remotes_data)
            }
            Db::MysqlDb(db) => {
                let remotes = db
                    .select_many_remotes_sync_by_address(&remote_address)
                    .await?;
                let mut remotes_data = Vec::with_capacity(remotes.len());
                for remote in &remotes {
                    remotes_data.push(Self::from_mysqldb_model(remote)?);
                }
                Ok(remotes_data)
            }
            Db::SqliteDb(db) => {
                let remotes = db
                    .select_many_remotes_sync_by_address(&remote_address)
                    .await?;
                let mut remotes_data = Vec::with_capacity(remotes.len());
                for remote in &remotes {
                    remotes_data.push(Self::from_sqlitedb_model(remote)?);
                }
                Ok(remotes_data)
            }
        }
    }

    fn from_postgresdb_model(model: &RemoteSyncPostgresModel) -> Result<Self> {
        Ok(Self {
            remote_address: SocketAddr::from_str(model.remote_address())?,
            remote_id: *model.remote_id(),
            last_data_sync: *model.last_data_sync(),
            last_change_id: *model.last_change_id(),
        })
    }

    fn to_postgresdb_model(&self) -> RemoteSyncPostgresModel {
        RemoteSyncPostgresModel::new(
            &self.remote_address.to_string(),
            &self.remote_id,
            &self.last_data_sync,
            &self.last_change_id,
        )
    }

    fn from_mysqldb_model(model: &RemoteSyncMysqlModel) -> Result<Self> {
        Ok(Self {
            remote_address: SocketAddr::from_str(model.remote_address())?,
            remote_id: *model.remote_id(),
            last_data_sync: *model.last_data_sync(),
            last_change_id: *model.last_change_id(),
        })
    }

    fn to_mysqldb_model(&self) -> RemoteSyncMysqlModel {
        RemoteSyncMysqlModel::new(
            &self.remote_address.to_string(),
            &self.remote_id,
            &self.last_data_sync,
            &self.last_change_id,
        )
    }

    fn from_sqlitedb_model(model: &RemoteSyncSqliteModel) -> Result<Self> {
        Ok(Self {
            remote_address: SocketAddr::from_str(model.remote_address())?,
            remote_id: *model.remote_id(),
            last_data_sync: *model.last_data_sync(),
            last_change_id: *model.last_change_id(),
        })
    }

    fn to_sqlitedb_model(&self) -> RemoteSyncSqliteModel {
        RemoteSyncSqliteModel::new(
            &self.remote_address.to_string(),
            &self.remote_id,
            &self.last_data_sync,
            &self.last_change_id,
        )
    }
}
