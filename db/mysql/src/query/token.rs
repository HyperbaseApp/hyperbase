use anyhow::Result;
use sqlx::{Executor, MySql, Pool};
use uuid::Uuid;

use crate::{db::MysqlDb, model::token::TokenModel};

const INSERT: &str = "INSERT INTO `tokens` (`id`, `created_at`, `updated_at`, `project_id`, `admin_id`, `name`, `token`, `allow_anonymous`, `expired_at`) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT `id`, `created_at`, `updated_at`, `project_id`, `admin_id`, `name`, `token`, `allow_anonymous`, `expired_at` FROM `tokens` WHERE `id` = ?";
const SELECT_MANY_BY_ADMIN_ID_AND_PROJECT_ID: &str = "SELECT `id`, `created_at`, `updated_at`, `project_id`, `admin_id`, `name`, `token`, `allow_anonymous`, `expired_at` FROM `tokens` WHERE `admin_id` = ? AND `project_id` = ? ORDER BY `id` DESC";
const SELECT_MANY_BY_PROJECT_ID: &str = "SELECT `id`, `created_at`, `updated_at`, `project_id`, `admin_id`, `name`, `token`, `allow_anonymous`, `expired_at` FROM `tokens` WHERE `project_id` = ? ORDER BY `id` DESC";
const UPDATE: &str = "UPDATE `tokens` SET `updated_at` = ?, `admin_id` = ?, `name` = ?, `allow_anonymous` = ?, `expired_at` = ? WHERE `id` = ?";
const DELETE: &str = "DELETE FROM `tokens` WHERE `id` = ?";

pub async fn init(pool: &Pool<MySql>) {
    hb_log::info(Some("🔧"), "[MySQL] Setting up tokens table");

    pool.execute("CREATE TABLE IF NOT EXISTS `tokens` (`id` binary(16), `created_at` timestamp(6), `updated_at` timestamp(6), `project_id` binary(16), `admin_id` binary(16), `name` text, `token` text, `allow_anonymous` boolean, `expired_at` timestamp(6), PRIMARY KEY (`id`))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT),
        pool.prepare(SELECT_MANY_BY_ADMIN_ID_AND_PROJECT_ID),
        pool.prepare(SELECT_MANY_BY_PROJECT_ID),
        pool.prepare(UPDATE),
        pool.prepare(DELETE),
    )
    .unwrap();
}

impl MysqlDb {
    pub async fn insert_token(&self, value: &TokenModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(value.id())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.project_id())
                .bind(value.admin_id())
                .bind(value.name())
                .bind(value.token())
                .bind(value.allow_anonymous())
                .bind(value.expired_at()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_token(&self, id: &Uuid) -> Result<TokenModel> {
        Ok(self.fetch_one(sqlx::query_as(SELECT).bind(id)).await?)
    }

    pub async fn select_many_tokens_by_admin_id_and_project_id(
        &self,
        admin_id: &Uuid,
        project_id: &Uuid,
    ) -> Result<Vec<TokenModel>> {
        Ok(self
            .fetch_all(
                sqlx::query_as(SELECT_MANY_BY_ADMIN_ID_AND_PROJECT_ID)
                    .bind(admin_id)
                    .bind(project_id),
            )
            .await?)
    }

    pub async fn select_many_tokens_by_project_id(
        &self,
        project_id: &Uuid,
    ) -> Result<Vec<TokenModel>> {
        Ok(self
            .fetch_all(sqlx::query_as(SELECT_MANY_BY_PROJECT_ID).bind(project_id))
            .await?)
    }

    pub async fn update_token(&self, value: &TokenModel) -> Result<()> {
        self.execute(
            sqlx::query(UPDATE)
                .bind(value.updated_at())
                .bind(value.admin_id())
                .bind(value.name())
                .bind(value.allow_anonymous())
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
