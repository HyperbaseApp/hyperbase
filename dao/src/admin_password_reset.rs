use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_mysql::model::admin_password_reset::AdminPasswordResetModel as AdminPasswordResetMysqlModel;
use hb_db_postgresql::model::admin_password_reset::AdminPasswordResetModel as AdminPasswordResetPostgresModel;
use hb_db_scylladb::model::admin_password_reset::AdminPasswordResetModel as AdminPasswordResetScyllaModel;
use hb_db_sqlite::model::admin_password_reset::AdminPasswordResetModel as AdminPasswordResetSqliteModel;
use rand::{thread_rng, Rng};
use uuid::Uuid;

use crate::{util::conversion, Db};

pub struct AdminPasswordResetDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    admin_id: Uuid,
    code: String,
}

impl AdminPasswordResetDao {
    pub fn new(admin_id: &Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            created_at: now,
            updated_at: now,
            admin_id: *admin_id,
            code: thread_rng().gen_range(100000..=999999).to_string(),
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

    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => {
                db.insert_admin_password_reset(&self.to_scylladb_model())
                    .await
            }
            Db::PostgresqlDb(db) => {
                db.insert_admin_password_reset(&self.to_postgresdb_model())
                    .await
            }
            Db::MysqlDb(db) => {
                db.insert_admin_password_reset(&self.to_mysqldb_model())
                    .await
            }
            Db::SqliteDb(db) => {
                db.insert_admin_password_reset(&self.to_sqlitedb_model())
                    .await
            }
        }
    }

    pub async fn db_select(db: &Db, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => {
                Self::from_scylladb_model(&db.select_admin_password_reset(id).await?)
            }
            Db::PostgresqlDb(db) => Ok(Self::from_postgresdb_model(
                &db.select_admin_password_reset(id).await?,
            )),
            Db::MysqlDb(db) => Ok(Self::from_mysqldb_model(
                &db.select_admin_password_reset(id).await?,
            )),
            Db::SqliteDb(db) => Ok(Self::from_sqlitedb_model(
                &db.select_admin_password_reset(id).await?,
            )),
        }
    }

    fn from_scylladb_model(model: &AdminPasswordResetScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.created_at())?,
            updated_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.updated_at())?,
            admin_id: *model.admin_id(),
            code: model.code().to_owned(),
        })
    }

    fn to_scylladb_model(&self) -> AdminPasswordResetScyllaModel {
        AdminPasswordResetScyllaModel::new(
            &self.id,
            &conversion::datetime_utc_to_scylla_cql_timestamp(&self.created_at),
            &conversion::datetime_utc_to_scylla_cql_timestamp(&self.updated_at),
            &self.admin_id,
            &self.code,
        )
    }

    fn from_postgresdb_model(model: &AdminPasswordResetPostgresModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            code: model.code().to_owned(),
        }
    }

    fn to_postgresdb_model(&self) -> AdminPasswordResetPostgresModel {
        AdminPasswordResetPostgresModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.admin_id,
            &self.code,
        )
    }

    fn from_mysqldb_model(model: &AdminPasswordResetMysqlModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            code: model.code().to_owned(),
        }
    }

    fn to_mysqldb_model(&self) -> AdminPasswordResetMysqlModel {
        AdminPasswordResetMysqlModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.admin_id,
            &self.code,
        )
    }

    fn from_sqlitedb_model(model: &AdminPasswordResetSqliteModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            code: model.code().to_owned(),
        }
    }

    fn to_sqlitedb_model(&self) -> AdminPasswordResetSqliteModel {
        AdminPasswordResetSqliteModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.admin_id,
            &self.code,
        )
    }
}
