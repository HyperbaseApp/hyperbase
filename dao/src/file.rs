use std::{path::Path, str::FromStr};

use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_mysql::model::file::FileModel as FileMysqlModel;
use hb_db_postgresql::model::file::FileModel as FilePostgresModel;
use hb_db_scylladb::model::file::FileModel as FileScyllaModel;
use hb_db_sqlite::model::file::FileModel as FileSqliteModel;
use mime::Mime;
use scylla::frame::value::CqlTimestamp as ScyllaCqlTimestamp;
use tokio::fs;
use uuid::Uuid;

use crate::{util::conversion, Db};

pub struct FileDao {
    id: Uuid,
    created_by: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    bucket_id: Uuid,
    file_name: String,
    content_type: Mime,
    size: i64,
}

impl FileDao {
    pub fn new(
        created_by: &Uuid,
        bucket_id: &Uuid,
        file_name: &str,
        content_type: &Mime,
        size: &i64,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::now_v7(),
            created_by: *created_by,
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

    pub fn created_by(&self) -> &Uuid {
        &self.created_by
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

    pub fn set_created_by(&mut self, created_by: &Uuid) {
        self.created_by = *created_by;
    }

    pub fn set_file_name(&mut self, file_name: &str) {
        self.file_name = file_name.to_owned();
    }

    pub async fn save(&self, db: &Db, bucket_path: &str, path: impl AsRef<Path>) -> Result<()> {
        fs::copy(path, &format!("{}/{}", bucket_path, &self.id)).await?;

        self.db_insert(db).await
    }

    pub async fn delete(db: &Db, bucket_path: &str, id: &Uuid) -> Result<()> {
        fs::remove_file(&format!("{}/{}", bucket_path, id)).await?;

        Self::db_delete(db, id).await
    }

    async fn db_insert(&self, db: &Db) -> Result<()> {
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

    pub async fn db_select_many_by_bucket_id(
        db: &Db,
        bucket_id: &Uuid,
        before_id: &Option<Uuid>,
        limit: &Option<i32>,
    ) -> Result<(Vec<Self>, i64)> {
        match db {
            Db::ScyllaDb(db) => {
                let mut files_data = Vec::new();
                let (files, total) = tokio::try_join!(
                    db.select_many_files_by_bucket_id(bucket_id, before_id, limit),
                    db.count_many_files_by_bucket_id(bucket_id)
                )?;
                for file in files {
                    files_data.push(Self::from_scylladb_model(&file?)?);
                }
                Ok((files_data, total))
            }
            Db::PostgresqlDb(db) => {
                let (files, total) = tokio::try_join!(
                    db.select_many_files_by_bucket_id(bucket_id, before_id, limit),
                    db.count_many_files_by_bucket_id(bucket_id)
                )?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_postgresdb_model(file)?)
                }
                Ok((files_data, total))
            }
            Db::MysqlDb(db) => {
                let (files, total) = tokio::try_join!(
                    db.select_many_files_by_bucket_id(bucket_id, before_id, limit),
                    db.count_many_files_by_bucket_id(bucket_id)
                )?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_mysqldb_model(file)?)
                }
                Ok((files_data, total))
            }
            Db::SqliteDb(db) => {
                let (files, total) = tokio::try_join!(
                    db.select_many_files_by_bucket_id(bucket_id, before_id, limit),
                    db.count_many_files_by_bucket_id(bucket_id)
                )?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_sqlitedb_model(file)?)
                }
                Ok((files_data, total))
            }
        }
    }

    pub async fn db_select_many_by_created_by_and_bucket_id(
        db: &Db,
        created_by: &Uuid,
        bucket_id: &Uuid,
        before_id: &Option<Uuid>,
        limit: &Option<i32>,
    ) -> Result<(Vec<Self>, i64)> {
        match db {
            Db::ScyllaDb(db) => {
                let mut files_data = Vec::new();
                let (files, total) = tokio::try_join!(
                    db.select_many_files_by_created_by_and_bucket_id(
                        created_by, bucket_id, before_id, limit,
                    ),
                    db.count_many_files_by_created_by_and_bucket_id(created_by, bucket_id)
                )?;
                for file in files {
                    files_data.push(Self::from_scylladb_model(&file?)?);
                }
                Ok((files_data, total))
            }
            Db::PostgresqlDb(db) => {
                let (files, total) = tokio::try_join!(
                    db.select_many_files_by_created_by_and_bucket_id(
                        created_by, bucket_id, before_id, limit,
                    ),
                    db.count_many_files_by_created_by_and_bucket_id(created_by, bucket_id)
                )?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_postgresdb_model(file)?)
                }
                Ok((files_data, total))
            }
            Db::MysqlDb(db) => {
                let (files, total) = tokio::try_join!(
                    db.select_many_files_by_created_by_and_bucket_id(
                        created_by, bucket_id, before_id, limit,
                    ),
                    db.count_many_files_by_created_by_and_bucket_id(created_by, bucket_id)
                )?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_mysqldb_model(file)?)
                }
                Ok((files_data, total))
            }
            Db::SqliteDb(db) => {
                let (files, total) = tokio::try_join!(
                    db.select_many_files_by_created_by_and_bucket_id(
                        created_by, bucket_id, before_id, limit,
                    ),
                    db.count_many_files_by_created_by_and_bucket_id(created_by, bucket_id)
                )?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_sqlitedb_model(file)?)
                }
                Ok((files_data, total))
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

    async fn db_delete(db: &Db, id: &Uuid) -> Result<()> {
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
            created_by: *model.created_by(),
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
            &self.created_by,
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
            created_by: *model.created_by(),
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
            &self.created_by,
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
            created_by: *model.created_by(),
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
            &self.created_by,
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
            created_by: *model.created_by(),
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
            &self.created_by,
            &self.created_at,
            &self.updated_at,
            &self.bucket_id,
            &self.file_name,
            &self.content_type.to_string(),
            &self.size,
        )
    }
}
