use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_mysql::{
    db::MysqlDb,
    model::project::ProjectModel as ProjectMysqlModel,
    query::project::{
        DELETE as MYSQL_DELETE, INSERT as MYSQL_INSERT, SELECT as MYSQL_SELECT,
        SELECT_MANY_BY_ADMIN_ID as MYSQL_SELECT_MANY_BY_ADMIN_ID, UPDATE as MYSQL_UPDATE,
    },
};
use hb_db_postgresql::{
    db::PostgresDb,
    model::project::ProjectModel as ProjectPostgresModel,
    query::project::{
        DELETE as POSTGRES_DELETE, INSERT as POSTGRES_INSERT, SELECT as POSTGRES_SELECT,
        SELECT_MANY_BY_ADMIN_ID as POSTGRES_SELECT_MANY_BY_ADMIN_ID, UPDATE as POSTGRES_UPDATE,
    },
};
use hb_db_scylladb::{
    db::ScyllaDb,
    model::project::ProjectModel as ProjectScyllaModel,
    query::project::{
        DELETE as SCYLLA_DELETE, INSERT as SCYLLA_INSERT, SELECT as SCYLLA_SELECT,
        SELECT_MANY_BY_ADMIN_ID as SCYLLA_SELECT_MANY_BY_ADMIN_ID, UPDATE as SCYLLA_UPDATE,
    },
};
use hb_db_sqlite::{
    db::SqliteDb,
    model::project::ProjectModel as ProjectSqliteModel,
    query::project::{
        DELETE as SQLITE_DELETE, INSERT as SQLITE_INSERT, SELECT as SQLITE_SELECT,
        SELECT_MANY_BY_ADMIN_ID as SQLITE_SELECT_MANY_BY_ADMIN_ID, UPDATE as SQLITE_UPDATE,
    },
};
use scylla::{
    frame::value::CqlTimestamp as ScyllaCqlTimestamp,
    transport::session::TypedRowIter as ScyllaTypedRowIter,
};
use uuid::Uuid;

use crate::{util::conversion, Db};

pub struct ProjectDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    admin_id: Uuid,
    name: String,
}

impl ProjectDao {
    pub fn new(admin_id: &Uuid, name: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            created_at: now,
            updated_at: now,
            admin_id: *admin_id,
            name: name.to_owned(),
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
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

    pub async fn db_select_many_by_admin_id(db: &Db, admin_id: &Uuid) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                let mut projects_data = Vec::new();
                let projects = Self::scylladb_select_many_by_admin_id(db, admin_id).await?;
                for project in projects {
                    projects_data.push(Self::from_scylladb_model(&project?)?);
                }
                Ok(projects_data)
            }
            Db::PostgresqlDb(db) => {
                let projects = Self::postgresdb_select_many_by_admin_id(db, admin_id).await?;
                let mut projects_data = Vec::with_capacity(projects.len());
                for project in &projects {
                    projects_data.push(Self::from_postgresdb_model(project)?);
                }
                Ok(projects_data)
            }
            Db::MysqlDb(db) => {
                let projects = Self::mysqldb_select_many_by_admin_id(db, admin_id).await?;
                let mut projects_data = Vec::with_capacity(projects.len());
                for project in &projects {
                    projects_data.push(Self::from_mysqldb_model(project)?);
                }
                Ok(projects_data)
            }
            Db::SqliteDb(db) => {
                let projects = Self::sqlitedb_select_many_by_admin_id(db, admin_id).await?;
                let mut projects_data = Vec::with_capacity(projects.len());
                for project in &projects {
                    projects_data.push(Self::from_sqlitedb_model(project)?);
                }
                Ok(projects_data)
            }
        }
    }

    pub async fn db_update(&mut self, db: &Db) -> Result<()> {
        self.updated_at = Utc::now();
        match db {
            Db::ScyllaDb(db) => Self::scylladb_update(self, db).await,
            Db::PostgresqlDb(db) => Self::postgresdb_update(self, db).await,
            Db::MysqlDb(db) => Self::mysqldb_update(self, db).await,
            Db::SqliteDb(db) => Self::sqlitedb_update(self, db).await,
        }
    }

    pub async fn db_delete(db: &Db, id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_delete(db, id).await,
            Db::PostgresqlDb(db) => Self::postgresdb_delete(db, id).await,
            Db::MysqlDb(db) => Self::mysqldb_delete(db, id).await,
            Db::SqliteDb(db) => Self::sqlitedb_delete(db, id).await,
        }
    }

    async fn scylladb_insert(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(SCYLLA_INSERT, &self.to_scylladb_model()).await?;
        Ok(())
    }

    async fn scylladb_select(db: &ScyllaDb, id: &Uuid) -> Result<ProjectScyllaModel> {
        Ok(db
            .execute(SCYLLA_SELECT, [id].as_ref())
            .await?
            .first_row_typed::<ProjectScyllaModel>()?)
    }

    async fn scylladb_select_many_by_admin_id(
        db: &ScyllaDb,
        admin_id: &Uuid,
    ) -> Result<ScyllaTypedRowIter<ProjectScyllaModel>> {
        Ok(db
            .execute(SCYLLA_SELECT_MANY_BY_ADMIN_ID, [admin_id].as_ref())
            .await?
            .rows_typed()?)
    }

    async fn scylladb_update(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(
            SCYLLA_UPDATE,
            &(
                &ScyllaCqlTimestamp(self.updated_at.timestamp_millis()),
                &self.name,
                &self.id,
            ),
        )
        .await?;
        Ok(())
    }

    async fn scylladb_delete(db: &ScyllaDb, id: &Uuid) -> Result<()> {
        db.execute(SCYLLA_DELETE, [id].as_ref()).await?;
        Ok(())
    }

    async fn postgresdb_insert(&self, db: &PostgresDb) -> Result<()> {
        db.execute(
            sqlx::query(POSTGRES_INSERT)
                .bind(&self.id)
                .bind(&self.created_at)
                .bind(&self.updated_at)
                .bind(&self.admin_id)
                .bind(&self.name),
        )
        .await?;
        Ok(())
    }

    async fn postgresdb_select(db: &PostgresDb, id: &Uuid) -> Result<ProjectPostgresModel> {
        Ok(db
            .fetch_one(sqlx::query_as(POSTGRES_SELECT).bind(id))
            .await?)
    }

    async fn postgresdb_select_many_by_admin_id(
        db: &PostgresDb,
        admin_id: &Uuid,
    ) -> Result<Vec<ProjectPostgresModel>> {
        Ok(db
            .fetch_all(sqlx::query_as(POSTGRES_SELECT_MANY_BY_ADMIN_ID).bind(admin_id))
            .await?)
    }

    async fn postgresdb_update(&self, db: &PostgresDb) -> Result<()> {
        db.execute(
            sqlx::query(POSTGRES_UPDATE)
                .bind(&self.updated_at)
                .bind(&self.name)
                .bind(&self.id),
        )
        .await?;
        Ok(())
    }

    async fn postgresdb_delete(db: &PostgresDb, id: &Uuid) -> Result<()> {
        db.execute(sqlx::query(POSTGRES_DELETE).bind(id)).await?;
        Ok(())
    }

    async fn mysqldb_insert(&self, db: &MysqlDb) -> Result<()> {
        db.execute(
            sqlx::query(MYSQL_INSERT)
                .bind(&self.id)
                .bind(&self.created_at)
                .bind(&self.updated_at)
                .bind(&self.admin_id)
                .bind(&self.name),
        )
        .await?;
        Ok(())
    }

    async fn mysqldb_select(db: &MysqlDb, id: &Uuid) -> Result<ProjectMysqlModel> {
        Ok(db.fetch_one(sqlx::query_as(MYSQL_SELECT).bind(id)).await?)
    }

    async fn mysqldb_select_many_by_admin_id(
        db: &MysqlDb,
        admin_id: &Uuid,
    ) -> Result<Vec<ProjectMysqlModel>> {
        Ok(db
            .fetch_all(sqlx::query_as(MYSQL_SELECT_MANY_BY_ADMIN_ID).bind(admin_id))
            .await?)
    }

    async fn mysqldb_update(&self, db: &MysqlDb) -> Result<()> {
        db.execute(
            sqlx::query(MYSQL_UPDATE)
                .bind(&self.updated_at)
                .bind(&self.name)
                .bind(&self.id),
        )
        .await?;
        Ok(())
    }

    async fn mysqldb_delete(db: &MysqlDb, id: &Uuid) -> Result<()> {
        db.execute(sqlx::query(MYSQL_DELETE).bind(id)).await?;
        Ok(())
    }

    async fn sqlitedb_insert(&self, db: &SqliteDb) -> Result<()> {
        db.execute(
            sqlx::query(SQLITE_INSERT)
                .bind(&self.id)
                .bind(&self.created_at)
                .bind(&self.updated_at)
                .bind(&self.admin_id)
                .bind(&self.name),
        )
        .await?;
        Ok(())
    }

    async fn sqlitedb_select(db: &SqliteDb, id: &Uuid) -> Result<ProjectSqliteModel> {
        Ok(db.fetch_one(sqlx::query_as(SQLITE_SELECT).bind(id)).await?)
    }

    async fn sqlitedb_select_many_by_admin_id(
        db: &SqliteDb,
        admin_id: &Uuid,
    ) -> Result<Vec<ProjectSqliteModel>> {
        Ok(db
            .fetch_all(sqlx::query_as(SQLITE_SELECT_MANY_BY_ADMIN_ID).bind(admin_id))
            .await?)
    }

    async fn sqlitedb_update(&self, db: &SqliteDb) -> Result<()> {
        db.execute(
            sqlx::query(SQLITE_UPDATE)
                .bind(&self.updated_at)
                .bind(&self.name)
                .bind(&self.id),
        )
        .await?;
        Ok(())
    }

    async fn sqlitedb_delete(db: &SqliteDb, id: &Uuid) -> Result<()> {
        db.execute(sqlx::query(SQLITE_DELETE).bind(id)).await?;
        Ok(())
    }

    fn from_scylladb_model(model: &ProjectScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.created_at())?,
            updated_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.updated_at())?,
            admin_id: *model.admin_id(),
            name: model.name().to_owned(),
        })
    }

    fn to_scylladb_model(&self) -> ProjectScyllaModel {
        ProjectScyllaModel::new(
            &self.id,
            &ScyllaCqlTimestamp(self.created_at.timestamp_millis()),
            &ScyllaCqlTimestamp(self.updated_at.timestamp_millis()),
            &self.admin_id,
            &self.name,
        )
    }

    fn from_postgresdb_model(model: &ProjectPostgresModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            name: model.name().to_owned(),
        })
    }

    fn from_mysqldb_model(model: &ProjectMysqlModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            name: model.name().to_owned(),
        })
    }

    fn from_sqlitedb_model(model: &ProjectSqliteModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            name: model.name().to_owned(),
        })
    }
}
