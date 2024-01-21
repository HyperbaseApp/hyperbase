use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_mysql::model::admin::AdminModel as AdminMysqlModel;
use hb_db_postgresql::model::admin::AdminModel as AdminPostgresModel;
use hb_db_scylladb::model::admin::AdminModel as AdminScyllaModel;
use hb_db_sqlite::model::admin::AdminModel as AdminSqliteModel;
use scylla::frame::value::CqlTimestamp as ScyllaCqlTimestamp;
use uuid::Uuid;

use crate::{util::conversion, Db};

pub struct AdminDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    email: String,
    password_hash: String,
}

impl AdminDao {
    pub fn new(email: &str, password_hash: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            created_at: now,
            updated_at: now,
            email: email.to_owned(),
            password_hash: password_hash.to_owned(),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn set_email(&mut self, email: &str) {
        self.email = email.to_owned()
    }

    pub fn set_password_hash(&mut self, password_hash: &str) {
        self.password_hash = password_hash.to_owned();
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.insert_admin(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.insert_admin(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.insert_admin(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.insert_admin(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_select(db: &Db, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Self::from_scylladb_model(&db.select_admin(id).await?),
            Db::PostgresqlDb(db) => Ok(Self::from_postgresdb_model(&db.select_admin(id).await?)),
            Db::MysqlDb(db) => Ok(Self::from_mysqldb_model(&db.select_admin(id).await?)),
            Db::SqliteDb(db) => Ok(Self::from_sqlitedb_model(&db.select_admin(id).await?)),
        }
    }

    pub async fn db_select_by_email(db: &Db, email: &str) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Self::from_scylladb_model(&db.select_admin_by_email(email).await?),
            Db::PostgresqlDb(db) => Ok(Self::from_postgresdb_model(
                &db.select_admin_by_email(email).await?,
            )),
            Db::MysqlDb(db) => Ok(Self::from_mysqldb_model(
                &db.select_admin_by_email(email).await?,
            )),
            Db::SqliteDb(db) => Ok(Self::from_sqlitedb_model(
                &db.select_admin_by_email(email).await?,
            )),
        }
    }

    pub async fn db_update(&mut self, db: &Db) -> Result<()> {
        self.updated_at = Utc::now();
        match db {
            Db::ScyllaDb(db) => db.update_admin(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.update_admin(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.update_admin(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.update_admin(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_delete(db: &Db, id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.delete_admin(id).await,
            Db::PostgresqlDb(db) => db.delete_admin(id).await,
            Db::MysqlDb(db) => db.delete_admin(id).await,
            Db::SqliteDb(db) => db.delete_admin(id).await,
        }
    }

    fn from_scylladb_model(model: &AdminScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.created_at())?,
            updated_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.updated_at())?,
            email: model.email().to_owned(),
            password_hash: model.password_hash().to_owned(),
        })
    }

    fn to_scylladb_model(&self) -> AdminScyllaModel {
        AdminScyllaModel::new(
            &self.id,
            &ScyllaCqlTimestamp(self.created_at.timestamp_millis()),
            &ScyllaCqlTimestamp(self.updated_at.timestamp_millis()),
            &self.email,
            &self.password_hash,
        )
    }

    fn from_postgresdb_model(model: &AdminPostgresModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            email: model.email().to_owned(),
            password_hash: model.password_hash().to_owned(),
        }
    }

    fn to_postgresdb_model(&self) -> AdminPostgresModel {
        AdminPostgresModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.email,
            &self.password_hash,
        )
    }

    fn from_mysqldb_model(model: &AdminMysqlModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            email: model.email().to_owned(),
            password_hash: model.password_hash().to_owned(),
        }
    }

    fn to_mysqldb_model(&self) -> AdminMysqlModel {
        AdminMysqlModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.email,
            &self.password_hash,
        )
    }

    fn from_sqlitedb_model(model: &AdminSqliteModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            email: model.email().to_owned(),
            password_hash: model.password_hash().to_owned(),
        }
    }

    fn to_sqlitedb_model(&self) -> AdminSqliteModel {
        AdminSqliteModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.email,
            &self.password_hash,
        )
    }
}
