use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use hb_db_mysql::model::change::ChangeModel as ChangeMysqlModel;
use hb_db_postgresql::model::change::ChangeModel as ChangePostgresModel;
use hb_db_scylladb::model::change::ChangeModel as ChangeScyllaModel;
use hb_db_sqlite::model::change::ChangeModel as ChangeSqliteModel;
use uuid::Uuid;

use crate::{util::conversion, Db};

pub struct ChangeDao {
    table: ChangeTable,
    id: Uuid,
    state: ChangeState,
    updated_at: DateTime<Utc>,
    change_id: Uuid,
}

impl ChangeDao {
    pub fn new(table: &ChangeTable, id: &Uuid, state: &ChangeState) -> Self {
        Self {
            table: *table,
            id: *id,
            state: *state,
            updated_at: Utc::now(),
            change_id: Uuid::now_v7(),
        }
    }

    pub fn table(&self) -> &ChangeTable {
        &self.table
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn state(&self) -> &ChangeState {
        &self.state
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    pub fn change_id(&self) -> &Uuid {
        &self.change_id
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.insert_change(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.insert_change(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.insert_change(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.insert_change(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_select_last(db: &Db) -> Result<Option<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                if let Some(model) = db.select_last_change().await? {
                    Ok(Some(Self::from_scylladb_model(&model)?))
                } else {
                    Ok(None)
                }
            }
            Db::PostgresqlDb(db) => {
                if let Some(model) = db.select_last_change().await? {
                    Ok(Some(Self::from_postgresdb_model(&model)?))
                } else {
                    Ok(None)
                }
            }
            Db::MysqlDb(db) => {
                if let Some(model) = db.select_last_change().await? {
                    Ok(Some(Self::from_mysqldb_model(&model)?))
                } else {
                    Ok(None)
                }
            }
            Db::SqliteDb(db) => {
                if let Some(model) = db.select_last_change().await? {
                    Ok(Some(Self::from_sqlitedb_model(&model)?))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub async fn db_select_many(db: &Db) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                let mut changes_data = Vec::new();
                let changes = db.select_many_changes().await?;
                for change in changes {
                    changes_data.push(Self::from_scylladb_model(&change?)?);
                }
                Ok(changes_data)
            }
            Db::PostgresqlDb(db) => {
                let changes = db.select_many_changes().await?;
                let mut changes_data = Vec::with_capacity(changes.len());
                for change in &changes {
                    changes_data.push(Self::from_postgresdb_model(change)?);
                }
                Ok(changes_data)
            }
            Db::MysqlDb(db) => {
                let changes = db.select_many_changes().await?;
                let mut changes_data = Vec::with_capacity(changes.len());
                for change in &changes {
                    changes_data.push(Self::from_mysqldb_model(change)?);
                }
                Ok(changes_data)
            }
            Db::SqliteDb(db) => {
                let changes = db.select_many_changes().await?;
                let mut changes_data = Vec::with_capacity(changes.len());
                for change in &changes {
                    changes_data.push(Self::from_sqlitedb_model(change)?);
                }
                Ok(changes_data)
            }
        }
    }

    pub async fn db_select_many_from_time(db: &Db, time: &DateTime<Utc>) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                let mut changes_data = Vec::new();
                let changes = db
                    .select_many_changes_from_time(
                        &conversion::datetime_utc_to_scylla_cql_timestamp(time),
                    )
                    .await?;
                for change in changes {
                    changes_data.push(Self::from_scylladb_model(&change?)?);
                }
                Ok(changes_data)
            }
            Db::PostgresqlDb(db) => {
                let changes = db.select_many_changes().await?;
                let mut changes_data = Vec::with_capacity(changes.len());
                for change in &changes {
                    changes_data.push(Self::from_postgresdb_model(change)?);
                }
                Ok(changes_data)
            }
            Db::MysqlDb(db) => {
                let changes = db.select_many_changes().await?;
                let mut changes_data = Vec::with_capacity(changes.len());
                for change in &changes {
                    changes_data.push(Self::from_mysqldb_model(change)?);
                }
                Ok(changes_data)
            }
            Db::SqliteDb(db) => {
                let changes = db.select_many_changes().await?;
                let mut changes_data = Vec::with_capacity(changes.len());
                for change in &changes {
                    changes_data.push(Self::from_sqlitedb_model(change)?);
                }
                Ok(changes_data)
            }
        }
    }

    pub async fn db_update(&mut self, db: &Db) -> Result<()> {
        self.updated_at = Utc::now();
        self.change_id = Uuid::now_v7();
        match db {
            Db::ScyllaDb(db) => {
                db.delete_change(self.table.to_str(), &self.id).await?;
                db.insert_change(&self.to_scylladb_model()).await
            }
            Db::PostgresqlDb(db) => db.update_change(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.update_change(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.update_change(&self.to_sqlitedb_model()).await,
        }
    }

    fn from_scylladb_model(model: &ChangeScyllaModel) -> Result<Self> {
        Ok(Self {
            table: ChangeTable::from_str(model.table())?,
            id: *model.id(),
            state: ChangeState::from_str(model.state())?,
            updated_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.updated_at())?,
            change_id: *model.change_id(),
        })
    }

    fn to_scylladb_model(&self) -> ChangeScyllaModel {
        ChangeScyllaModel::new(
            self.table.to_str(),
            &self.id,
            self.state.to_str(),
            &conversion::datetime_utc_to_scylla_cql_timestamp(&self.updated_at),
            &self.change_id,
        )
    }

    fn from_postgresdb_model(model: &ChangePostgresModel) -> Result<Self> {
        Ok(Self {
            table: ChangeTable::from_str(model.table())?,
            id: *model.id(),
            state: ChangeState::from_str(model.state())?,
            updated_at: *model.updated_at(),
            change_id: *model.change_id(),
        })
    }

    fn to_postgresdb_model(&self) -> ChangePostgresModel {
        ChangePostgresModel::new(
            self.table.to_str(),
            &self.id,
            self.state.to_str(),
            &self.updated_at,
            &self.change_id,
        )
    }

    fn from_mysqldb_model(model: &ChangeMysqlModel) -> Result<Self> {
        Ok(Self {
            table: ChangeTable::from_str(model.table())?,
            id: *model.id(),
            state: ChangeState::from_str(model.state())?,
            updated_at: *model.updated_at(),
            change_id: *model.change_id(),
        })
    }

    fn to_mysqldb_model(&self) -> ChangeMysqlModel {
        ChangeMysqlModel::new(
            self.table.to_str(),
            &self.id,
            self.state.to_str(),
            &self.updated_at,
            &self.change_id,
        )
    }

    fn from_sqlitedb_model(model: &ChangeSqliteModel) -> Result<Self> {
        Ok(Self {
            table: ChangeTable::from_str(model.table())?,
            id: *model.id(),
            state: ChangeState::from_str(model.state())?,
            updated_at: *model.updated_at(),
            change_id: *model.change_id(),
        })
    }

    fn to_sqlitedb_model(&self) -> ChangeSqliteModel {
        ChangeSqliteModel::new(
            self.table.to_str(),
            &self.id,
            self.state.to_str(),
            &self.updated_at,
            &self.change_id,
        )
    }
}

#[derive(Clone, Copy)]
pub enum ChangeTable {
    Admin,
    Project,
    Collection,
    Record,
    Bucket,
    File,
    Token,
    CollectionRule,
    BucketRule,
    Log,
}

impl ChangeTable {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Admin => "admins",
            Self::Project => "projects",
            Self::Collection => "collections",
            Self::Record => "record",
            Self::Bucket => "buckets",
            Self::File => "files",
            Self::Token => "tokens",
            Self::CollectionRule => "collection_rules",
            Self::BucketRule => "bucket_rules",
            Self::Log => "logs",
        }
    }

    pub fn from_str(str: &str) -> Result<Self> {
        match str {
            "admins" => Ok(Self::Admin),
            "projects" => Ok(Self::Project),
            "collections" => Ok(Self::Collection),
            "records" => Ok(Self::Record),
            "buckets" => Ok(Self::Bucket),
            "files" => Ok(Self::File),
            "tokens" => Ok(Self::Token),
            "collection_rules" => Ok(Self::CollectionRule),
            "bucket_rules" => Ok(Self::BucketRule),
            "logs" => Ok(Self::Log),
            _ => Err(Error::msg(format!("Unknown change table '{str}'"))),
        }
    }
}

#[derive(Clone, Copy)]
pub enum ChangeState {
    Insert,
    Update,
    Delete,
}

impl ChangeState {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Insert => "insert",
            Self::Update => "update",
            Self::Delete => "delete",
        }
    }

    pub fn from_str(str: &str) -> Result<Self> {
        match str {
            "insert" => Ok(Self::Insert),
            "update" => Ok(Self::Update),
            "delete" => Ok(Self::Delete),
            _ => Err(Error::msg(format!("Unknown change state '{str}'"))),
        }
    }
}
