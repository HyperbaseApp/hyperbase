use std::str::FromStr;

use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_mysql::model::file::FileModel as FileMysqlModel;
use hb_db_postgresql::model::file::FileModel as FilePostgresModel;
use hb_db_scylladb::model::file::FileModel as FileScyllaModel;
use hb_db_sqlite::model::file::FileModel as FileSqliteModel;
use mime::Mime;
use scylla::frame::value::CqlTimestamp as ScyllaCqlTimestamp;
use uuid::Uuid;

use crate::{util::conversion, Db};

pub struct FileDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    bucket_id: Uuid,
    file_name: String,
    content_type: Mime,
    size: i64,
}

impl FileDao {
    pub fn new(bucket_id: &Uuid, file_name: &str, content_type: &Mime, size: &i64) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::now_v7(),
            created_at: now,
            updated_at: now,
            bucket_id: *bucket_id,
            file_name: file_name.to_owned(),
            content_type: content_type.clone(),
            size: *size,
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

    pub fn bucket_id(&self) -> &Uuid {
        &self.bucket_id
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn content_type(&self) -> &Mime {
        &self.content_type
    }

    pub fn size(&self) -> &i64 {
        &self.size
    }

    pub fn set_file_name(&mut self, file_name: &str) {
        self.file_name = file_name.to_owned();
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.insert_file(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.insert_file(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.insert_file(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.insert_file(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_select(db: &Db, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Self::from_scylladb_model(&db.select_file(id).await?),
            Db::PostgresqlDb(db) => Self::from_postgresdb_model(&db.select_file(id).await?),
            Db::MysqlDb(db) => Self::from_mysqldb_model(&db.select_file(id).await?),
            Db::SqliteDb(db) => Self::from_sqlitedb_model(&db.select_file(id).await?),
        }
    }

    pub async fn db_select_many_by_bucket_id(db: &Db, bucket_id: &Uuid) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                let mut files_data = Vec::new();
                let files = db.select_many_files_by_bucket_id(bucket_id).await?;
                for file in files {
                    files_data.push(Self::from_scylladb_model(&file?)?);
                }
                Ok(files_data)
            }
            Db::PostgresqlDb(db) => {
                let files = db.select_many_files_by_bucket_id(bucket_id).await?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_postgresdb_model(file)?)
                }
                Ok(files_data)
            }
            Db::MysqlDb(db) => {
                let files = db.select_many_files_by_bucket_id(bucket_id).await?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_mysqldb_model(file)?)
                }
                Ok(files_data)
            }
            Db::SqliteDb(db) => {
                let files = db.select_many_files_by_bucket_id(bucket_id).await?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_sqlitedb_model(file)?)
                }
                Ok(files_data)
            }
        }
    }

    pub async fn db_update(&mut self, db: &Db) -> Result<()> {
        self.updated_at = Utc::now();
        match db {
            Db::ScyllaDb(db) => db.update_file(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.update_file(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.update_file(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.update_file(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_delete(db: &Db, id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.delete_file(id).await,
            Db::PostgresqlDb(db) => db.delete_file(id).await,
            Db::MysqlDb(db) => db.delete_file(id).await,
            Db::SqliteDb(db) => db.delete_file(id).await,
        }
    }

    fn from_scylladb_model(model: &FileScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.created_at())?,
            updated_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.updated_at())?,
            bucket_id: *model.bucket_id(),
            file_name: model.file_name().to_owned(),
            content_type: Mime::from_str(model.content_type())?,
            size: *model.size(),
        })
    }

    fn to_scylladb_model(&self) -> FileScyllaModel {
        FileScyllaModel::new(
            &self.id,
            &ScyllaCqlTimestamp(self.created_at.timestamp_millis()),
            &ScyllaCqlTimestamp(self.updated_at.timestamp_millis()),
            &self.bucket_id,
            &self.file_name,
            &self.content_type.to_string(),
            &self.size,
        )
    }

    fn from_postgresdb_model(model: &FilePostgresModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            bucket_id: *model.bucket_id(),
            file_name: model.file_name().to_owned(),
            content_type: Mime::from_str(model.content_type())?,
            size: *model.size(),
        })
    }

    fn to_postgresdb_model(&self) -> FilePostgresModel {
        FilePostgresModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.bucket_id,
            &self.file_name,
            &self.content_type.to_string(),
            &self.size,
        )
    }

    fn from_mysqldb_model(model: &FileMysqlModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            bucket_id: *model.bucket_id(),
            file_name: model.file_name().to_owned(),
            content_type: Mime::from_str(model.content_type())?,
            size: *model.size(),
        })
    }

    fn to_mysqldb_model(&self) -> FileMysqlModel {
        FileMysqlModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.bucket_id,
            &self.file_name,
            &self.content_type.to_string(),
            &self.size,
        )
    }

    fn from_sqlitedb_model(model: &FileSqliteModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            bucket_id: *model.bucket_id(),
            file_name: model.file_name().to_owned(),
            content_type: Mime::from_str(model.content_type())?,
            size: *model.size(),
        })
    }

    fn to_sqlitedb_model(&self) -> FileSqliteModel {
        FileSqliteModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.bucket_id,
            &self.file_name,
            &self.content_type.to_string(),
            &self.size,
        )
    }
}
