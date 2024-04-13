use anyhow::{Error, Result};
use chrono::{Duration, Utc};
use sqlx::{Executor, Pool, Sqlite};
use uuid::Uuid;

use crate::{db::SqliteDb, model::file::FileModel};

const INSERT: &str = "INSERT INTO \"files\" (\"id\", \"created_by\", \"created_at\", \"updated_at\", \"bucket_id\", \"file_name\", \"content_type\", \"size\") VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT \"id\", \"created_by\", \"created_at\", \"updated_at\", \"bucket_id\", \"file_name\", \"content_type\", \"size\" FROM \"files\" WHERE \"id\" = ?";
const SELECT_MANY_BY_BUCKET_ID: &str = "SELECT \"id\", \"created_by\", \"created_at\", \"updated_at\", \"bucket_id\", \"file_name\", \"content_type\", \"size\" FROM \"files\" WHERE \"bucket_id\" = ?";
const COUNT_MANY_BY_BUCKET_ID: &str = "SELECT COUNT(1) FROM \"files\" WHERE \"bucket_id\" = ?";
const SELECT_MANY_BY_CREATED_BY_AND_BUCKET_ID: &str = "SELECT \"id\", \"created_by\", \"created_at\", \"updated_at\", \"bucket_id\", \"file_name\", \"content_type\", \"size\" FROM \"files\" WHERE \"created_by\" = ? AND \"bucket_id\" = ?";
const COUNT_MANY_BY_CREATED_BY_AND_BUCKET_ID: &str = "SELECT COUNT(1) FROM \"files\" WHERE \"created_by\" = ? AND \"bucket_id\" = ?";
const SELECT_MANY_EXPIRE: &str = "SELECT \"id\", \"created_by\", \"created_at\", \"updated_at\", \"bucket_id\", \"file_name\", \"content_type\", \"size\" FROM \"files\" WHERE \"updated_at\" < ?";
const UPDATE: &str = "UPDATE \"files\" SET \"created_by\" = ?, \"updated_at\" = ?, \"file_name\" = ? WHERE \"id\" = ?";
const DELETE: &str = "DELETE FROM \"files\" WHERE \"id\" = ?";

pub async fn init(pool: &Pool<Sqlite>) {
    hb_log::info(Some("🔧"), "SQLite: Setting up files table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"files\" (\"id\" blob, \"created_by\" blob, \"created_at\" timestamp, \"updated_at\" timestamp, \"bucket_id\" blob, \"file_name\" text, \"content_type\" text, \"size\" integer, PRIMARY KEY (\"id\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT),
        pool.prepare(SELECT_MANY_BY_BUCKET_ID),
        pool.prepare(COUNT_MANY_BY_BUCKET_ID),
        pool.prepare(SELECT_MANY_BY_CREATED_BY_AND_BUCKET_ID),
        pool.prepare(COUNT_MANY_BY_CREATED_BY_AND_BUCKET_ID),
        pool.prepare(SELECT_MANY_EXPIRE),
        pool.prepare(UPDATE),
        pool.prepare(DELETE),
    )
    .unwrap();
}

impl SqliteDb {
    pub async fn insert_file(&self, value: &FileModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(value.id())
                .bind(value.created_by())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.bucket_id())
                .bind(value.file_name())
                .bind(value.content_type())
                .bind(value.size()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_file(&self, id: &Uuid) -> Result<FileModel> {
        Ok(self.fetch_one(sqlx::query_as(SELECT).bind(id)).await?)
    }

    pub async fn select_many_files_by_bucket_id(
        &self,
        bucket_id: &Uuid,
        before_id: &Option<Uuid>,
        limit: &Option<i32>,
    ) -> Result<Vec<FileModel>> {
        let mut sql = SELECT_MANY_BY_BUCKET_ID.to_owned();
        if before_id.is_some() {
            sql += " AND \"id\" < ?";
        }
        sql += " ORDER BY \"id\" DESC";
        if limit.is_some() {
            sql += " LIMIT ?";
        }

        let mut query = sqlx::query_as(&sql).bind(bucket_id);
        if let Some(before_id) = before_id {
            query = query.bind(before_id);
        }
        if let Some(limit) = limit {
            query = query.bind(limit);
        }

        Ok(self.fetch_all(query).await?)
    }

    pub async fn count_many_files_by_bucket_id(&self, bucket_id: &Uuid) -> Result<i64> {
        Ok(self
            .fetch_one::<(i64,)>(sqlx::query_as(COUNT_MANY_BY_BUCKET_ID).bind(bucket_id))
            .await?
            .0)
    }

    pub async fn select_many_files_by_created_by_and_bucket_id(
        &self,
        created_by: &Uuid,
        bucket_id: &Uuid,
        before_id: &Option<Uuid>,
        limit: &Option<i32>,
    ) -> Result<Vec<FileModel>> {
        let mut sql = SELECT_MANY_BY_CREATED_BY_AND_BUCKET_ID.to_owned();
        if before_id.is_some() {
            sql += " AND \"id\" < ?";
        }
        sql += " ORDER BY \"id\" DESC";
        if limit.is_some() {
            sql += " LIMIT ?";
        }

        let mut query = sqlx::query_as(&sql).bind(created_by).bind(bucket_id);
        if let Some(before_id) = before_id {
            query = query.bind(before_id);
        }
        if let Some(limit) = limit {
            query = query.bind(limit);
        }

        Ok(self.fetch_all(query).await?)
    }

    pub async fn count_many_files_by_created_by_and_bucket_id(
        &self,
        created_by: &Uuid,
        bucket_id: &Uuid,
    ) -> Result<i64> {
        Ok(self
            .fetch_one::<(i64,)>(
                sqlx::query_as(COUNT_MANY_BY_CREATED_BY_AND_BUCKET_ID)
                    .bind(created_by)
                    .bind(bucket_id),
            )
            .await?
            .0)
    }

    pub async fn select_many_expired_file(&self, ttl_seconds: &i64) -> Result<Vec<FileModel>> {
        Ok(self
            .fetch_all(
                sqlx::query_as(SELECT_MANY_EXPIRE).bind(
                    Utc::now()
                        .checked_sub_signed(
                            Duration::try_seconds(*ttl_seconds)
                                .ok_or_else(|| Error::msg("bucket ttl is out of range."))?,
                        )
                        .ok_or_else(|| Error::msg("bucket ttl is out of range."))?,
                ),
            )
            .await?)
    }

    pub async fn update_file(&self, value: &FileModel) -> Result<()> {
        self.execute(
            sqlx::query(UPDATE)
                .bind(value.created_by())
                .bind(value.updated_at())
                .bind(value.file_name())
                .bind(value.id()),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_file(&self, id: &Uuid) -> Result<()> {
        self.execute(sqlx::query(DELETE).bind(id)).await?;
        Ok(())
    }
}
