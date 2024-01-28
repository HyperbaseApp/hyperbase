use anyhow::Result;
use sqlx::{Executor, MySql, Pool};
use uuid::Uuid;

use crate::{db::MysqlDb, model::token::TokenModel};

const INSERT: &str = "INSERT INTO `tokens` (`id`, `created_at`, `updated_at`, `admin_id`, `token`, `bucket_rules`, `collection_rules`, `expired_at`) VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT `id`, `created_at`, `updated_at`, `admin_id`, `token`, `bucket_rules`, `collection_rules`, `expired_at` FROM `tokens` WHERE `id` = ?";
const SELECT_MANY_BY_ADMIN_ID: &str = "SELECT `id`, `created_at`, `updated_at`, `admin_id`, `token`, `bucket_rules`, `collection_rules`, `expired_at` FROM `tokens` WHERE `admin_id` = ?";
const UPDATE: &str = "UPDATE `tokens` SET `updated_at` = ?, `bucket_rules` = ?, `collection_rules` = ?, `expired_at` = ? WHERE `id` = ?";
const DELETE: &str = "DELETE FROM `tokens` WHERE `id` = ?";

pub async fn init(pool: &Pool<MySql>) {
    hb_log::info(Some("🔧"), "MySQL: Setting up tokens table");

    pool.execute("CREATE TABLE IF NOT EXISTS `tokens` (`id` binary(16), `created_at` timestamp, `updated_at` timestamp, `admin_id` binary(16), `token` text, `bucket_rules` json, `collection_rules` json, `expired_at` timestamp, PRIMARY KEY (`id`))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(SELECT_MANY_BY_ADMIN_ID).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}

impl MysqlDb {
    pub async fn insert_token(&self, value: &TokenModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(value.id())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.admin_id())
                .bind(value.token())
                .bind(value.bucket_rules())
                .bind(value.collection_rules())
                .bind(value.expired_at()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_token(&self, id: &Uuid) -> Result<TokenModel> {
        Ok(self.fetch_one(sqlx::query_as(SELECT).bind(id)).await?)
    }

    pub async fn select_many_tokens_by_admin_id(&self, admin_id: &Uuid) -> Result<Vec<TokenModel>> {
        Ok(self
            .fetch_all(sqlx::query_as(SELECT_MANY_BY_ADMIN_ID).bind(admin_id))
            .await?)
    }

    pub async fn update_token(&self, value: &TokenModel) -> Result<()> {
        self.execute(
            sqlx::query(UPDATE)
                .bind(value.updated_at())
                .bind(&sqlx::types::Json(value.bucket_rules()))
                .bind(&sqlx::types::Json(value.collection_rules()))
                .bind(value.expired_at())
                .bind(value.id()),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_token(&self, id: &Uuid) -> Result<()> {
        self.execute(sqlx::query(DELETE).bind(id)).await?;
        Ok(())
    }
}
