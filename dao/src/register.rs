use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_mysql::{
    db::MysqlDb,
    model::registration::RegistrationModel as RegistrationMysqlModel,
    query::registration::{DELETE as MYSQL_DELETE, INSERT as MYSQL_INSERT, SELECT as MYSQL_SELECT},
};
use hb_db_postgresql::{
    db::PostgresDb,
    model::registration::RegistrationModel as RegistrationPostgresModel,
    query::registration::{
        DELETE as POSTGRES_DELETE, INSERT as POSTGRES_INSERT, SELECT as POSTGRES_SELECT,
    },
};
use hb_db_scylladb::{
    db::ScyllaDb,
    model::registration::RegistrationModel as RegistrationScyllaModel,
    query::registration::{
        DELETE as SCYLLA_DELETE, INSERT as SCYLLA_INSERT, SELECT as SCYLLA_SELECT,
    },
};
use hb_db_sqlite::{
    db::SqliteDb,
    model::registration::RegistrationModel as RegistrationSqliteModel,
    query::registration::{
        DELETE as SQLITE_DELETE, INSERT as SQLITE_INSERT, SELECT as SQLITE_SELECT,
    },
};
use rand::{thread_rng, Rng};
use scylla::frame::value::CqlTimestamp as ScyllaCqlTimestamp;
use uuid::Uuid;

use crate::{util::conversion, Db};

pub struct RegistrationDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    email: String,
    password_hash: String,
    code: String,
}

impl RegistrationDao {
    pub fn new(email: &str, password_hash: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            email: email.to_owned(),
            password_hash: password_hash.to_owned(),
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

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_insert(self, db).await,
            Db::PostgresqlDb(db) => Self::postgresdb_insert(self, db).await,
            Db::MysqlDb(db) => Self::mysqldb_insert(self, db).await,
            Db::SqliteDb(db) => Self::sqlitedb_insert(self, db).await,
        }
    }

    pub async fn db_select(db: &Db, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Ok(Self::from_scylladb_model(
                &Self::scylladb_select(db, id).await?,
            )?),
            Db::PostgresqlDb(db) => Ok(Self::from_postgresdb_model(
                &Self::postgresdb_select(db, id).await?,
            )?),
            Db::MysqlDb(db) => Ok(Self::from_mysqldb_model(
                &Self::mysqldb_select(db, id).await?,
            )?),
            Db::SqliteDb(db) => Ok(Self::from_sqlitedb_model(
                &Self::sqlitedb_select(db, id).await?,
            )?),
        }
    }

    pub async fn db_delete(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_delete(self, db).await,
            Db::PostgresqlDb(db) => Self::postgresdb_delete(self, db).await,
            Db::MysqlDb(db) => Self::mysqldb_delete(self, db).await,
            Db::SqliteDb(db) => Self::sqlitedb_delete(self, db).await,
        }
    }

    async fn scylladb_insert(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(SCYLLA_INSERT, &self.to_scylladb_model()).await?;
        Ok(())
    }

    async fn scylladb_select(db: &ScyllaDb, id: &Uuid) -> Result<RegistrationScyllaModel> {
        Ok(db
            .execute(SCYLLA_SELECT, [id].as_ref())
            .await?
            .first_row_typed::<RegistrationScyllaModel>()?)
    }

    async fn scylladb_delete(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(SCYLLA_DELETE, [self.id()].as_ref()).await?;
        Ok(())
    }

    async fn postgresdb_insert(&self, db: &PostgresDb) -> Result<()> {
        db.execute(
            sqlx::query(POSTGRES_INSERT)
                .bind(&self.id)
                .bind(&self.created_at)
                .bind(&self.updated_at)
                .bind(&self.email)
                .bind(&self.password_hash)
                .bind(&self.code),
        )
        .await?;
        Ok(())
    }

    async fn postgresdb_select(db: &PostgresDb, id: &Uuid) -> Result<RegistrationPostgresModel> {
        Ok(db
            .fetch_one(sqlx::query_as(POSTGRES_SELECT).bind(id).bind(&{
                let now = Utc::now();
                DateTime::from_timestamp(
                    now.timestamp() - db.table_registration_ttl(),
                    now.timestamp_subsec_nanos(),
                )
                .unwrap()
            }))
            .await?)
    }

    async fn postgresdb_delete(&self, db: &PostgresDb) -> Result<()> {
        db.execute(sqlx::query(POSTGRES_DELETE).bind(&self.id))
            .await?;
        Ok(())
    }

    async fn mysqldb_insert(&self, db: &MysqlDb) -> Result<()> {
        db.execute(
            sqlx::query(MYSQL_INSERT)
                .bind(&self.id)
                .bind(&self.created_at)
                .bind(&self.updated_at)
                .bind(&self.email)
                .bind(&self.password_hash)
                .bind(&self.code),
        )
        .await?;
        Ok(())
    }

    async fn mysqldb_select(db: &MysqlDb, id: &Uuid) -> Result<RegistrationMysqlModel> {
        Ok(db
            .fetch_one(sqlx::query_as(MYSQL_SELECT).bind(id).bind(&{
                let now = Utc::now();
                DateTime::from_timestamp(
                    now.timestamp() - db.table_registration_ttl(),
                    now.timestamp_subsec_nanos(),
                )
                .unwrap()
            }))
            .await?)
    }

    async fn mysqldb_delete(&self, db: &MysqlDb) -> Result<()> {
        db.execute(sqlx::query(MYSQL_DELETE).bind(&self.id)).await?;
        Ok(())
    }

    async fn sqlitedb_insert(&self, db: &SqliteDb) -> Result<()> {
        db.execute(
            sqlx::query(SQLITE_INSERT)
                .bind(&self.id)
                .bind(&self.created_at)
                .bind(&self.updated_at)
                .bind(&self.email)
                .bind(&self.password_hash)
                .bind(&self.code),
        )
        .await?;
        Ok(())
    }

    async fn sqlitedb_select(db: &SqliteDb, id: &Uuid) -> Result<RegistrationSqliteModel> {
        Ok(db
            .fetch_one(sqlx::query_as(SQLITE_SELECT).bind(id).bind(&{
                let now = Utc::now();
                DateTime::from_timestamp(
                    now.timestamp() - db.table_registration_ttl(),
                    now.timestamp_subsec_nanos(),
                )
                .unwrap()
            }))
            .await?)
    }

    async fn sqlitedb_delete(&self, db: &SqliteDb) -> Result<()> {
        db.execute(sqlx::query(SQLITE_DELETE).bind(&self.id))
            .await?;
        Ok(())
    }

    fn from_scylladb_model(model: &RegistrationScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.created_at())?,
            updated_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.updated_at())?,
            email: model.email().to_owned(),
            password_hash: model.password_hash().to_owned(),
            code: model.code().to_owned(),
        })
    }

    fn to_scylladb_model(&self) -> RegistrationScyllaModel {
        RegistrationScyllaModel::new(
            &self.id,
            &ScyllaCqlTimestamp(self.created_at.timestamp_millis()),
            &ScyllaCqlTimestamp(self.updated_at.timestamp_millis()),
            &self.email,
            &self.password_hash,
            &self.code,
        )
    }

    fn from_postgresdb_model(model: &RegistrationPostgresModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            email: model.email().to_owned(),
            password_hash: model.password_hash().to_owned(),
            code: model.code().to_owned(),
        })
    }

    fn from_mysqldb_model(model: &RegistrationMysqlModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            email: model.email().to_owned(),
            password_hash: model.password_hash().to_owned(),
            code: model.code().to_owned(),
        })
    }

    fn from_sqlitedb_model(model: &RegistrationSqliteModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            email: model.email().to_owned(),
            password_hash: model.password_hash().to_owned(),
            code: model.code().to_owned(),
        })
    }
}
