use std::{net::SocketAddr, str::FromStr};

use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_mysql::model::remote_sync::RemoteSyncModel as RemoteSyncMysqlModel;
use hb_db_postgresql::model::remote_sync::RemoteSyncModel as RemoteSyncPostgresModel;
use hb_db_scylladb::model::remote_sync::RemoteSyncModel as RemoteSyncScyllaModel;
use hb_db_sqlite::model::remote_sync::RemoteSyncModel as RemoteSyncSqliteModel;
use uuid::Uuid;

use crate::{util::conversion, Db};

pub struct RemoteSyncDao {
    remote_address: SocketAddr,
    remote_id: Uuid,
    last_data_sync: DateTime<Utc>,
}

impl RemoteSyncDao {
    pub fn new(
        remote_address: &SocketAddr,
        remote_id: &Uuid,
        last_data_sync: &DateTime<Utc>,
    ) -> Self {
        Self {
            remote_address: *remote_address,
            remote_id: *remote_id,
            last_data_sync: *last_data_sync,
        }
    }

    pub fn remote_address(&self) -> &SocketAddr {
        &self.remote_address
    }

    pub fn remote_id(&self) -> &Uuid {
        &self.remote_id
    }

    pub fn last_data_sync(&self) -> &DateTime<Utc> {
        &self.last_data_sync
    }

    pub fn set_remote_id(&mut self, remote_id: &Uuid) {
        self.remote_id = *remote_id;
    }

    pub fn set_last_data_sync(&mut self, last_data_sync: &DateTime<Utc>) {
        self.last_data_sync = *last_data_sync;
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.insert_remote_sync(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.insert_remote_sync(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.insert_remote_sync(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.insert_remote_sync(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_select(db: &Db, remote_address: &SocketAddr) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Self::from_scylladb_model(
                &db.select_remote_sync(&remote_address.to_string()).await?,
            ),
            Db::PostgresqlDb(db) => Self::from_postgresdb_model(
                &db.select_remote_sync(&remote_address.to_string()).await?,
            ),
            Db::MysqlDb(db) => {
                Self::from_mysqldb_model(&db.select_remote_sync(&remote_address.to_string()).await?)
            }
            Db::SqliteDb(db) => Self::from_sqlitedb_model(
                &db.select_remote_sync(&remote_address.to_string()).await?,
            ),
        }
    }

    pub async fn db_update(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.update_remote_sync(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.update_remote_sync(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.update_remote_sync(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.update_remote_sync(&self.to_sqlitedb_model()).await,
        }
    }

    fn from_scylladb_model(model: &RemoteSyncScyllaModel) -> Result<Self> {
        Ok(Self {
            remote_address: SocketAddr::from_str(model.remote_address())?,
            remote_id: *model.remote_id(),
            last_data_sync: conversion::scylla_cql_timestamp_to_datetime_utc(
                model.last_data_sync(),
            )?,
        })
    }

    fn to_scylladb_model(&self) -> RemoteSyncScyllaModel {
        RemoteSyncScyllaModel::new(
            &self.remote_address.to_string(),
            &self.remote_id,
            &conversion::datetime_utc_to_scylla_cql_timestamp(&self.last_data_sync),
        )
    }

    fn from_postgresdb_model(model: &RemoteSyncPostgresModel) -> Result<Self> {
        Ok(Self {
            remote_address: SocketAddr::from_str(model.remote_address())?,
            remote_id: *model.remote_id(),
            last_data_sync: *model.last_data_sync(),
        })
    }

    fn to_postgresdb_model(&self) -> RemoteSyncPostgresModel {
        RemoteSyncPostgresModel::new(
            &self.remote_address.to_string(),
            &self.remote_id,
            &self.last_data_sync,
        )
    }

    fn from_mysqldb_model(model: &RemoteSyncMysqlModel) -> Result<Self> {
        Ok(Self {
            remote_address: SocketAddr::from_str(model.remote_address())?,
            remote_id: *model.remote_id(),
            last_data_sync: *model.last_data_sync(),
        })
    }

    fn to_mysqldb_model(&self) -> RemoteSyncMysqlModel {
        RemoteSyncMysqlModel::new(
            &self.remote_address.to_string(),
            &self.remote_id,
            &self.last_data_sync,
        )
    }

    fn from_sqlitedb_model(model: &RemoteSyncSqliteModel) -> Result<Self> {
        Ok(Self {
            remote_address: SocketAddr::from_str(model.remote_address())?,
            remote_id: *model.remote_id(),
            last_data_sync: *model.last_data_sync(),
        })
    }

    fn to_sqlitedb_model(&self) -> RemoteSyncSqliteModel {
        RemoteSyncSqliteModel::new(
            &self.remote_address.to_string(),
            &self.remote_id,
            &self.last_data_sync,
        )
    }
}
