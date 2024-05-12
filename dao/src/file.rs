use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use futures::future;
use hb_db_mysql::model::file::FileModel as FileMysqlModel;
use hb_db_postgresql::model::file::FileModel as FilePostgresModel;
use hb_db_scylladb::model::file::FileModel as FileScyllaModel;
use hb_db_sqlite::model::file::FileModel as FileSqliteModel;
use mime::Mime;
use serde::{Deserialize, Serialize};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};
use uuid::Uuid;

use crate::{bucket::BucketDao, util::conversion, Db};

#[derive(Deserialize, Serialize)]
pub struct FileDao {
    id: Uuid,
    created_by: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    bucket_id: Uuid,
    file_name: String,
    content_type: String,
    size: i64,
    public: bool,
    _bytes: Option<Vec<u8>>,
}

impl FileDao {
    pub fn new(
        created_by: &Uuid,
        bucket_id: &Uuid,
        file_name: &str,
        content_type: &Mime,
        size: &i64,
        public: &bool,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::now_v7(),
            created_by: *created_by,
            created_at: now,
            updated_at: now,
            bucket_id: *bucket_id,
            file_name: file_name.to_owned(),
            content_type: content_type.to_string(),
            size: *size,
            public: *public,
            _bytes: None,
        }
    }

    pub fn from_bytes<'a>(bytes: &'a [u8]) -> Result<Self, rmp_serde::decode::Error>
    where
        Self: Deserialize<'a>,
    {
        rmp_serde::from_slice(bytes)
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(self)
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

    pub fn content_type(&self) -> Mime {
        Mime::from_str(&self.content_type).unwrap()
    }

    pub fn size(&self) -> &i64 {
        &self.size
    }

    pub fn public(&self) -> &bool {
        &self.public
    }

    pub fn set_created_by(&mut self, created_by: &Uuid) {
        self.created_by = *created_by;
    }

    pub fn set_file_name(&mut self, file_name: &str) {
        self.file_name = file_name.to_owned();
    }

    pub fn set_public(&mut self, public: &bool) {
        self.public = *public;
    }

    pub async fn populate_file_bytes(&mut self, bucket_path: &str) -> Result<()> {
        let mut file = fs::File::open(&self.full_path(bucket_path)?).await?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).await?;
        self._bytes = Some(bytes);
        Ok(())
    }

    pub fn drop_bytes(&mut self) {
        self._bytes = None;
    }

    pub fn full_path(&self, bucket_path: &str) -> Result<PathBuf> {
        let exe_path = std::env::current_exe()?;
        let dir_path =
            match exe_path.parent() {
                Some(dir_path) => match dir_path.to_str() {
                    Some(path) => path,
                    None => return Err(Error::msg(
                        "Failed to convert directory path of the current executable as a string",
                    )),
                },
                None => {
                    return Err(Error::msg(
                        "Failed to get directory path of the current executable",
                    ))
                }
            };
        Ok(PathBuf::from(format!(
            "{}/{}/{}",
            dir_path, bucket_path, self.id
        )))
    }

    pub async fn save(&self, db: &Db, bucket_path: &str, path: impl AsRef<Path>) -> Result<()> {
        fs::copy(path, &self.full_path(bucket_path)?).await?;

        self.db_insert(db).await
    }

    pub async fn save_from_bytes(&self, db: &Db, bucket_path: &str) -> Result<()> {
        if let Some(bytes) = &self._bytes {
            let mut file = fs::File::create(self.full_path(bucket_path)?).await?;
            file.write_all(bytes).await?;
            file.flush().await?;

            self.db_insert(db).await
        } else {
            Err(Error::msg("File bytes is empty"))
        }
    }

    pub async fn delete(db: &Db, bucket_data: &BucketDao, id: &Uuid) -> Result<()> {
        let exe_path = std::env::current_exe()?;
        let exe_path = match exe_path.to_str() {
            Some(path) => path,
            None => return Err(Error::msg("Failed to get current executable path")),
        };
        fs::remove_file(&format!("{}/{}/{}", exe_path, bucket_data.path(), id)).await?;

        Self::db_delete(db, bucket_data.id(), id).await
    }

    async fn delete_expired(db: &Db, bucket_data: &BucketDao) -> Result<()> {
        if let Some(ttl_seconds) = bucket_data.opt_ttl() {
            let files_data =
                Self::db_select_many_expired(db, bucket_data.id(), ttl_seconds).await?;
            let mut delete_expired_mut = Vec::with_capacity(files_data.len());
            for file_data in &files_data {
                delete_expired_mut.push(Self::delete(db, bucket_data, &file_data.id));
            }
            future::try_join_all(delete_expired_mut).await?;
        }
        Ok(())
    }

    async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.insert_file(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.insert_file(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.insert_file(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.insert_file(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_select(db: &Db, bucket_data: &BucketDao, id: &Uuid) -> Result<Self> {
        Self::delete_expired(db, bucket_data).await?;

        match db {
            Db::ScyllaDb(db) => Self::from_scylladb_model(&db.select_file(id).await?),
            Db::PostgresqlDb(db) => Self::from_postgresdb_model(&db.select_file(id).await?),
            Db::MysqlDb(db) => Self::from_mysqldb_model(&db.select_file(id).await?),
            Db::SqliteDb(db) => Self::from_sqlitedb_model(&db.select_file(id).await?),
        }
    }

    pub async fn db_select_many_by_bucket_id(
        db: &Db,
        bucket_data: &BucketDao,
        before_id: &Option<Uuid>,
        limit: &Option<i32>,
    ) -> Result<(Vec<Self>, i64)> {
        Self::delete_expired(db, bucket_data).await?;

        match db {
            Db::ScyllaDb(db) => {
                let mut files_data = Vec::new();
                let (files, total) = tokio::try_join!(
                    db.select_many_files_by_bucket_id(bucket_data.id(), before_id, limit),
                    db.count_many_files_by_bucket_id(bucket_data.id())
                )?;
                for file in files {
                    files_data.push(Self::from_scylladb_model(&file?)?);
                }
                Ok((files_data, total))
            }
            Db::PostgresqlDb(db) => {
                let (files, total) = tokio::try_join!(
                    db.select_many_files_by_bucket_id(bucket_data.id(), before_id, limit),
                    db.count_many_files_by_bucket_id(bucket_data.id())
                )?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_postgresdb_model(file)?)
                }
                Ok((files_data, total))
            }
            Db::MysqlDb(db) => {
                let (files, total) = tokio::try_join!(
                    db.select_many_files_by_bucket_id(bucket_data.id(), before_id, limit),
                    db.count_many_files_by_bucket_id(bucket_data.id())
                )?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_mysqldb_model(file)?)
                }
                Ok((files_data, total))
            }
            Db::SqliteDb(db) => {
                let (files, total) = tokio::try_join!(
                    db.select_many_files_by_bucket_id(bucket_data.id(), before_id, limit),
                    db.count_many_files_by_bucket_id(bucket_data.id())
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
        bucket_data: &BucketDao,
        before_id: &Option<Uuid>,
        limit: &Option<i32>,
    ) -> Result<(Vec<Self>, i64)> {
        Self::delete_expired(db, bucket_data).await?;

        match db {
            Db::ScyllaDb(db) => {
                let mut files_data = Vec::new();
                let (files, total) = tokio::try_join!(
                    db.select_many_files_by_created_by_and_bucket_id(
                        created_by,
                        bucket_data.id(),
                        before_id,
                        limit,
                    ),
                    db.count_many_files_by_created_by_and_bucket_id(created_by, bucket_data.id())
                )?;
                for file in files {
                    files_data.push(Self::from_scylladb_model(&file?)?);
                }
                Ok((files_data, total))
            }
            Db::PostgresqlDb(db) => {
                let (files, total) = tokio::try_join!(
                    db.select_many_files_by_created_by_and_bucket_id(
                        created_by,
                        bucket_data.id(),
                        before_id,
                        limit,
                    ),
                    db.count_many_files_by_created_by_and_bucket_id(created_by, bucket_data.id())
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
                        created_by,
                        bucket_data.id(),
                        before_id,
                        limit,
                    ),
                    db.count_many_files_by_created_by_and_bucket_id(created_by, bucket_data.id())
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
                        created_by,
                        bucket_data.id(),
                        before_id,
                        limit,
                    ),
                    db.count_many_files_by_created_by_and_bucket_id(created_by, bucket_data.id())
                )?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_sqlitedb_model(file)?)
                }
                Ok((files_data, total))
            }
        }
    }

    async fn db_select_many_expired(
        db: &Db,
        bucket_id: &Uuid,
        ttl_seconds: &i64,
    ) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                let mut files_data = Vec::new();
                let files = db.select_many_expired_file(bucket_id, ttl_seconds).await?;
                for file in files {
                    files_data.push(Self::from_scylladb_model(&file?)?);
                }
                Ok(files_data)
            }
            Db::PostgresqlDb(db) => {
                let files = db.select_many_expired_file(bucket_id, ttl_seconds).await?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_postgresdb_model(file)?);
                }
                Ok(files_data)
            }
            Db::MysqlDb(db) => {
                let files = db.select_many_expired_file(bucket_id, ttl_seconds).await?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_mysqldb_model(file)?);
                }
                Ok(files_data)
            }
            Db::SqliteDb(db) => {
                let files = db.select_many_expired_file(bucket_id, ttl_seconds).await?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_sqlitedb_model(file)?);
                }
                Ok(files_data)
            }
        }
    }

    pub async fn db_select_many_from_updated_at_and_after_id_with_limit_asc(
        db: &Db,
        updated_at: &DateTime<Utc>,
        id: &Uuid,
        limit: &i32,
    ) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(_) => Err(Error::msg("Unimplemented")),
            Db::PostgresqlDb(db) => {
                let files = db
                    .select_many_files_from_updated_at_and_after_id_with_limit_asc(
                        updated_at, id, limit,
                    )
                    .await?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_postgresdb_model(file)?);
                }
                Ok(files_data)
            }
            Db::MysqlDb(db) => {
                let files = db
                    .select_many_files_from_updated_at_and_after_id_with_limit_asc(
                        updated_at, id, limit,
                    )
                    .await?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_mysqldb_model(file)?);
                }
                Ok(files_data)
            }
            Db::SqliteDb(db) => {
                let files = db
                    .select_many_files_from_updated_at_and_after_id_with_limit_asc(
                        updated_at, id, limit,
                    )
                    .await?;
                let mut files_data = Vec::with_capacity(files.len());
                for file in &files {
                    files_data.push(Self::from_sqlitedb_model(file)?);
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

    async fn db_delete(db: &Db, bucket_id: &Uuid, id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.delete_file(bucket_id, id).await,
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
            content_type: Mime::from_str(model.content_type())?.to_string(),
            size: *model.size(),
            public: *model.public(),
            _bytes: None,
        })
    }

    fn to_scylladb_model(&self) -> FileScyllaModel {
        FileScyllaModel::new(
            &self.id,
            &self.created_by,
            &conversion::datetime_utc_to_scylla_cql_timestamp(&self.created_at),
            &conversion::datetime_utc_to_scylla_cql_timestamp(&self.updated_at),
            &self.bucket_id,
            &self.file_name,
            &self.content_type.to_string(),
            &self.size,
            &self.public,
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
            content_type: Mime::from_str(model.content_type())?.to_string(),
            size: *model.size(),
            public: *model.public(),
            _bytes: None,
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
            &self.public,
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
            content_type: Mime::from_str(model.content_type())?.to_string(),
            size: *model.size(),
            public: *model.public(),
            _bytes: None,
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
            &self.public,
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
            content_type: Mime::from_str(model.content_type())?.to_string(),
            size: *model.size(),
            public: *model.public(),
            _bytes: None,
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
            &self.public,
        )
    }
}
