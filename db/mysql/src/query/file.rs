use anyhow::Result;
use sqlx::{Executor, MySql, Pool};
use uuid::Uuid;

use crate::{db::MysqlDb, model::file::FileModel};

const INSERT: &str = "INSERT INTO `files` (`id`, `created_at`, `updated_at`, `bucket_id`, `file_name`, `content_type`, `size`) VALUES (?, ?, ?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT `id`, `created_at`, `updated_at`, `bucket_id`, `file_name`, `content_type`, `size` FROM `files` WHERE `id` = ?";
const SELECT_MANY_BY_BUCKET_ID: &str = "SELECT `id`, `created_at`, `updated_at`, `bucket_id`, `file_name`, `content_type`, `size` FROM `files` WHERE `bucket_id` = ?";
const UPDATE: &str = "UPDATE `files` SET `updated_at` = ?, `file_name` = ? WHERE `id` = ?";
const DELETE: &str = "DELETE FROM `files` WHERE `id` = ?";

pub async fn init(pool: &Pool<MySql>) {
    hb_log::info(Some("🔧"), "MySQL: Setting up files table");

    pool.execute("CREATE TABLE IF NOT EXISTS `files` (`id` binary(16), `created_at` timestamp, `updated_at` timestamp, `bucket_id` binary(16), `file_name` text, `content_type` text, `size` bigint, PRIMARY KEY (`id`))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(SELECT_MANY_BY_BUCKET_ID).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}

impl MysqlDb {
    pub async fn insert_file(&self, value: &FileModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(value.id())
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

    pub async fn select_many_files_by_bucket_id(&self, bucket_id: &Uuid) -> Result<Vec<FileModel>> {
        Ok(self
            .fetch_all(sqlx::query_as(SELECT_MANY_BY_BUCKET_ID).bind(bucket_id))
            .await?)
    }

    pub async fn update_file(&self, value: &FileModel) -> Result<()> {
        self.execute(
            sqlx::query(UPDATE)
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
