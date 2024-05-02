use anyhow::Result;
use chrono::{DateTime, Utc};
use futures::future;
use hb_db_mysql::model::bucket::BucketModel as BucketMysqlModel;
use hb_db_postgresql::model::bucket::BucketModel as BucketPostgresModel;
use hb_db_scylladb::model::bucket::BucketModel as BucketScyllaModel;
use hb_db_sqlite::model::bucket::BucketModel as BucketSqliteModel;
use tokio::fs;
use uuid::Uuid;

use crate::{bucket_rule::BucketRuleDao, file::FileDao, util::conversion, Db};

pub struct BucketDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    project_id: Uuid,
    name: String,
    path: String,
    opt_ttl: Option<i64>,
}

impl BucketDao {
    pub async fn new(
        project_id: &Uuid,
        name: &str,
        path: &str,
        opt_ttl: &Option<i64>,
    ) -> Result<Self> {
        fs::create_dir_all(path).await?;

        let now = Utc::now();

        Ok(Self {
            id: Uuid::now_v7(),
            created_at: now,
            updated_at: now,
            project_id: *project_id,
            name: name.to_owned(),
            path: path.to_owned(),
            opt_ttl: *opt_ttl,
        })
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

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn opt_ttl(&self) -> &Option<i64> {
        &self.opt_ttl
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

    pub fn set_opt_ttl(&mut self, opt_ttl: &Option<i64>) {
        if let Some(opt_ttl) = opt_ttl {
            if *opt_ttl <= 0 {
                self.opt_ttl = None;
                return;
            }
        }
        self.opt_ttl = *opt_ttl;
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.insert_bucket(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.insert_bucket(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.insert_bucket(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.insert_bucket(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_select(db: &Db, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Self::from_scylladb_model(&db.select_bucket(id).await?),
            Db::PostgresqlDb(db) => Ok(Self::from_postgresdb_model(&db.select_bucket(id).await?)),
            Db::MysqlDb(db) => Ok(Self::from_mysqldb_model(&db.select_bucket(id).await?)),
            Db::SqliteDb(db) => Ok(Self::from_sqlitedb_model(&db.select_bucket(id).await?)),
        }
    }

    pub async fn db_select_many_by_project_id(db: &Db, project_id: &Uuid) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                let mut buckets_data = Vec::new();
                let buckets = db.select_many_buckets_by_project_id(project_id).await?;
                for bucket in buckets {
                    buckets_data.push(Self::from_scylladb_model(&bucket?)?);
                }
                Ok(buckets_data)
            }
            Db::PostgresqlDb(db) => {
                let buckets = db.select_many_buckets_by_project_id(project_id).await?;
                let mut buckets_data = Vec::with_capacity(buckets.len());
                for bucket in &buckets {
                    buckets_data.push(Self::from_postgresdb_model(bucket));
                }
                Ok(buckets_data)
            }
            Db::MysqlDb(db) => {
                let buckets = db.select_many_buckets_by_project_id(project_id).await?;
                let mut buckets_data = Vec::with_capacity(buckets.len());
                for bucket in &buckets {
                    buckets_data.push(Self::from_mysqldb_model(bucket));
                }
                Ok(buckets_data)
            }
            Db::SqliteDb(db) => {
                let buckets = db.select_many_buckets_by_project_id(project_id).await?;
                let mut buckets_data = Vec::with_capacity(buckets.len());
                for bucket in &buckets {
                    buckets_data.push(Self::from_sqlitedb_model(bucket));
                }
                Ok(buckets_data)
            }
        }
    }

    pub async fn db_update(&mut self, db: &Db) -> Result<()> {
        self.updated_at = Utc::now();
        match db {
            Db::ScyllaDb(db) => db.update_bucket(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.update_bucket(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.update_bucket(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.update_bucket(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_delete(db: &Db, id: &Uuid) -> Result<()> {
        let bucket_data = Self::db_select(db, id).await?;

        let (files_data, _) =
            FileDao::db_select_many_by_bucket_id(db, &bucket_data, &None, &None).await?;
        let mut delete_file_mut = Vec::with_capacity(files_data.len());
        for file_data in &files_data {
            delete_file_mut.push(FileDao::delete(db, &bucket_data, file_data.id()));
        }
        future::try_join_all(delete_file_mut).await?;

        BucketRuleDao::db_delete_many_by_bucket_id(db, id).await?;

        match db {
            Db::ScyllaDb(db) => db.delete_bucket(id).await,
            Db::PostgresqlDb(db) => db.delete_bucket(id).await,
            Db::MysqlDb(db) => db.delete_bucket(id).await,
            Db::SqliteDb(db) => db.delete_bucket(id).await,
        }
    }

    fn from_scylladb_model(model: &BucketScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.created_at())?,
            updated_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.updated_at())?,
            project_id: *model.project_id(),
            name: model.name().to_owned(),
            path: model.path().to_owned(),
            opt_ttl: *model.opt_ttl(),
        })
    }

    fn to_scylladb_model(&self) -> BucketScyllaModel {
        BucketScyllaModel::new(
            &self.id,
            &conversion::datetime_utc_to_scylla_cql_timestamp(&self.created_at),
            &conversion::datetime_utc_to_scylla_cql_timestamp(&self.updated_at),
            &self.project_id,
            &self.name,
            &self.path,
            &self.opt_ttl,
        )
    }

    fn from_postgresdb_model(model: &BucketPostgresModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            project_id: *model.project_id(),
            name: model.name().to_owned(),
            path: model.path().to_owned(),
            opt_ttl: *model.opt_ttl(),
        }
    }

    fn to_postgresdb_model(&self) -> BucketPostgresModel {
        BucketPostgresModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.project_id,
            &self.name,
            &self.path,
            &self.opt_ttl,
        )
    }

    fn from_mysqldb_model(model: &BucketMysqlModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            project_id: *model.project_id(),
            name: model.name().to_owned(),
            path: model.path().to_owned(),
            opt_ttl: *model.opt_ttl(),
        }
    }

    fn to_mysqldb_model(&self) -> BucketMysqlModel {
        BucketMysqlModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.project_id,
            &self.name,
            &self.path,
            &self.opt_ttl,
        )
    }

    fn from_sqlitedb_model(model: &BucketSqliteModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            project_id: *model.project_id(),
            name: model.name().to_owned(),
            path: model.path().to_owned(),
            opt_ttl: *model.opt_ttl(),
        }
    }

    fn to_sqlitedb_model(&self) -> BucketSqliteModel {
        BucketSqliteModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.project_id,
            &self.name,
            &self.path,
            &self.opt_ttl,
        )
    }
}
