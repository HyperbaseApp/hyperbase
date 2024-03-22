use anyhow::Result;
use chrono::{DateTime, Utc};
use futures::future;
use hb_db_mysql::model::project::ProjectModel as ProjectMysqlModel;
use hb_db_postgresql::model::project::ProjectModel as ProjectPostgresModel;
use hb_db_scylladb::model::project::ProjectModel as ProjectScyllaModel;
use hb_db_sqlite::model::project::ProjectModel as ProjectSqliteModel;
use scylla::frame::value::CqlTimestamp as ScyllaCqlTimestamp;
use uuid::Uuid;

use crate::{bucket::BucketDao, collection::CollectionDao, token::TokenDao, util::conversion, Db};

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
            Db::ScyllaDb(db) => db.insert_project(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.insert_project(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.insert_project(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.insert_project(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_select(db: &Db, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Self::from_scylladb_model(&db.select_project(id).await?),
            Db::PostgresqlDb(db) => Ok(Self::from_postgresdb_model(&db.select_project(id).await?)),
            Db::MysqlDb(db) => Ok(Self::from_mysqldb_model(&db.select_project(id).await?)),
            Db::SqliteDb(db) => Ok(Self::from_sqlitedb_model(&db.select_project(id).await?)),
        }
    }

    pub async fn db_select_many_by_admin_id(db: &Db, admin_id: &Uuid) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                let mut projects_data = Vec::new();
                let projects = db.select_many_projects_by_admin_id(admin_id).await?;
                for project in projects {
                    projects_data.push(Self::from_scylladb_model(&project?)?);
                }
                Ok(projects_data)
            }
            Db::PostgresqlDb(db) => {
                let projects = db.select_many_projects_by_admin_id(admin_id).await?;
                let mut projects_data = Vec::with_capacity(projects.len());
                for project in &projects {
                    projects_data.push(Self::from_postgresdb_model(project));
                }
                Ok(projects_data)
            }
            Db::MysqlDb(db) => {
                let projects = db.select_many_projects_by_admin_id(admin_id).await?;
                let mut projects_data = Vec::with_capacity(projects.len());
                for project in &projects {
                    projects_data.push(Self::from_mysqldb_model(project));
                }
                Ok(projects_data)
            }
            Db::SqliteDb(db) => {
                let projects = db.select_many_projects_by_admin_id(admin_id).await?;
                let mut projects_data = Vec::with_capacity(projects.len());
                for project in &projects {
                    projects_data.push(Self::from_sqlitedb_model(project));
                }
                Ok(projects_data)
            }
        }
    }

    pub async fn db_update(&mut self, db: &Db) -> Result<()> {
        self.updated_at = Utc::now();
        match db {
            Db::ScyllaDb(db) => db.update_project(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.update_project(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.update_project(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.update_project(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_delete(db: &Db, id: &Uuid) -> Result<()> {
        let (collections_data, buckets_data, tokens_data) = tokio::try_join!(
            CollectionDao::db_select_many_by_project_id(db, id),
            BucketDao::db_select_many_by_project_id(db, id),
            TokenDao::db_select_many_by_project_id(db, id)
        )?;

        let mut remove_collections = Vec::with_capacity(collections_data.len());
        for collection_data in &collections_data {
            remove_collections.push(CollectionDao::db_delete(db, collection_data.id()));
        }
        future::join_all(remove_collections).await;

        let mut remove_buckets = Vec::with_capacity(buckets_data.len());
        for bucket_data in &buckets_data {
            remove_buckets.push(BucketDao::db_delete(db, bucket_data.id()));
        }
        future::join_all(remove_buckets).await;

        let mut remove_tokens = Vec::with_capacity(tokens_data.len());
        for token_data in &tokens_data {
            remove_tokens.push(TokenDao::db_delete(db, token_data.id()));
        }
        future::join_all(remove_tokens).await;

        match db {
            Db::ScyllaDb(db) => db.delete_project(id).await,
            Db::PostgresqlDb(db) => db.delete_project(id).await,
            Db::MysqlDb(db) => db.delete_project(id).await,
            Db::SqliteDb(db) => db.delete_project(id).await,
        }
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

    fn from_postgresdb_model(model: &ProjectPostgresModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            name: model.name().to_owned(),
        }
    }

    fn to_postgresdb_model(&self) -> ProjectPostgresModel {
        ProjectPostgresModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.admin_id,
            &self.name,
        )
    }

    fn from_mysqldb_model(model: &ProjectMysqlModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            name: model.name().to_owned(),
        }
    }

    fn to_mysqldb_model(&self) -> ProjectMysqlModel {
        ProjectMysqlModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.admin_id,
            &self.name,
        )
    }

    fn from_sqlitedb_model(model: &ProjectSqliteModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            name: model.name().to_owned(),
        }
    }

    fn to_sqlitedb_model(&self) -> ProjectSqliteModel {
        ProjectSqliteModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.admin_id,
            &self.name,
        )
    }
}
