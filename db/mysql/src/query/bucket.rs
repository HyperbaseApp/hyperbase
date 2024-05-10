use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Executor, MySql, Pool};
use uuid::Uuid;

use crate::{db::MysqlDb, model::bucket::BucketModel};

const INSERT: &str = "INSERT INTO `buckets` (`id`, `created_at`, `updated_at`, `project_id`, `name`, `path`, `opt_ttl`) VALUES (?, ?, ?, ?, ?, ?, ?)";
const UPSERT: &str = "INSERT INTO `buckets` (`id`, `created_at`, `updated_at`, `project_id`, `name`, `path`, `opt_ttl`) VALUES (?, ?, ?, ?, ?, ?, ?) ON DUPLICATE KEY UPDATE `created_at` = ?, `updated_at` = ?, `project_id` = ?, `name` = ?, `path` = ?, `opt_ttl` = ?";
const SELECT: &str = "SELECT `id`, `created_at`, `updated_at`, `project_id`, `name`, `path`, `opt_ttl` FROM `buckets` WHERE `id` = ?";
const SELECT_MANY_BY_PROJECT_ID: &str = "SELECT `id`, `created_at`, `updated_at`, `project_id`, `name`, `path`, `opt_ttl` FROM `buckets` WHERE `project_id` = ? ORDER BY `id` DESC";
const SELECT_MANY_FROM_UPDATED_AT_AND_AFTER_ID_WITH_LIMIT_ASC: &str = "SELECT `id`, `created_at`, `updated_at`, `project_id`, `name`, `path`, `opt_ttl` FROM `buckets` WHERE `updated_at` > ? OR (`updated_at` = ? AND `id` > ?) ORDER BY `updated_at` ASC, `id` ASC LIMIT ?";
const UPDATE: &str = "UPDATE `buckets` SET `updated_at` = ?, `name` = ?, `opt_ttl` = ? WHERE `id` = ?";
const DELETE: &str = "DELETE FROM `buckets` WHERE `id` = ?";

pub async fn init(pool: &Pool<MySql>) {
    hb_log::info(Some("🔧"), "[MySQL] Setting up buckets table");

    pool.execute("CREATE TABLE IF NOT EXISTS `buckets` (`id` binary(16), `created_at` timestamp, `updated_at` timestamp, `project_id` binary(16), `name` text, `path` text, `opt_ttl` bigint, PRIMARY KEY (`id`))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(UPSERT),
        pool.prepare(SELECT),
        pool.prepare(SELECT_MANY_BY_PROJECT_ID),
        pool.prepare(SELECT_MANY_FROM_UPDATED_AT_AND_AFTER_ID_WITH_LIMIT_ASC),
        pool.prepare(UPDATE),
        pool.prepare(DELETE),
    )
    .unwrap();
}

impl MysqlDb {
    pub async fn insert_bucket(&self, value: &BucketModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(value.id())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.project_id())
                .bind(value.name())
                .bind(value.path())
                .bind(value.opt_ttl()),
        )
        .await?;
        Ok(())
    }

    pub async fn upsert_bucket(&self, value: &BucketModel) -> Result<()> {
        self.execute(
            sqlx::query(UPSERT)
                .bind(value.id())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.project_id())
                .bind(value.name())
                .bind(value.path())
                .bind(value.opt_ttl())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.project_id())
                .bind(value.name())
                .bind(value.path())
                .bind(value.opt_ttl()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_bucket(&self, id: &Uuid) -> Result<BucketModel> {
        Ok(self.fetch_one(sqlx::query_as(SELECT).bind(id)).await?)
    }

    pub async fn select_many_buckets_by_project_id(
        &self,
        project_id: &Uuid,
    ) -> Result<Vec<BucketModel>> {
        Ok(self
            .fetch_all(sqlx::query_as(SELECT_MANY_BY_PROJECT_ID).bind(project_id))
            .await?)
    }

    pub async fn select_many_buckets_from_updated_at_and_after_id_with_limit_asc(
        &self,
        updated_at: &DateTime<Utc>,
        id: &Uuid,
        limit: &i32,
    ) -> Result<Vec<BucketModel>> {
        Ok(self
            .fetch_all(
                sqlx::query_as(SELECT_MANY_FROM_UPDATED_AT_AND_AFTER_ID_WITH_LIMIT_ASC)
                    .bind(updated_at)
                    .bind(updated_at)
                    .bind(id)
                    .bind(limit),
            )
            .await?)
    }

    pub async fn update_bucket(&self, value: &BucketModel) -> Result<()> {
        self.execute(
            sqlx::query(UPDATE)
                .bind(value.updated_at())
                .bind(value.name())
                .bind(value.opt_ttl())
                .bind(value.id()),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_bucket(&self, id: &Uuid) -> Result<()> {
        self.execute(sqlx::query(DELETE).bind(id)).await?;
        Ok(())
    }
}
