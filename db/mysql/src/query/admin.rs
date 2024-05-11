use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Executor, MySql, Pool};
use uuid::Uuid;

use crate::{db::MysqlDb, model::admin::AdminModel};

const INSERT: &str = "INSERT INTO `admins` (`id`, `created_at`, `updated_at`, `email`, `password_hash`) VALUES (?, ?, ?, ?, ?)";
const UPSERT: &str = "INSERT INTO `admins` (`id`, `created_at`, `updated_at`, `email`, `password_hash`) VALUES (?, ?, ?, ?, ?) ON DUPLICATE KEY UPDATE `created_at` = ?, `updated_at` = ?, `email` = ?, `password_hash` = ?";
const SELECT: &str = "SELECT `id`, `created_at`, `updated_at`, `email`, `password_hash` FROM `admins` WHERE `id` = ?";
const SELECT_BY_EMAIL: &str= "SELECT `id`, `created_at`, `updated_at`, `email`, `password_hash` FROM `admins` WHERE `email` = ?";
const SELECT_MANY_FROM_UPDATED_AT_AND_AFTER_ID_WITH_LIMIT_ASC: &str = "SELECT `id`, `created_at`, `updated_at`, `email`, `password_hash` FROM `admins` WHERE `updated_at` > ? OR (`updated_at` = ? AND `id` > ?) ORDER BY `updated_at` ASC, `id` ASC LIMIT ?";
const UPDATE: &str = "UPDATE `admins` SET `updated_at` = ?, `email` = ?, `password_hash` = ? WHERE `id` = ?";
const DELETE: &str = "DELETE FROM `admins` WHERE `id` = ?";

pub async fn init(pool: &Pool<MySql>) {
    hb_log::info(Some("🔧"), "[MySQL] Setting up admins table");

    pool.execute("CREATE TABLE IF NOT EXISTS `admins` (`id` binary(16), `created_at` timestamp(6), `updated_at` timestamp(6), `email` text, `password_hash` text, PRIMARY KEY (`id`))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(UPSERT),
        pool.prepare(SELECT),
        pool.prepare(SELECT_BY_EMAIL),
        pool.prepare(SELECT_MANY_FROM_UPDATED_AT_AND_AFTER_ID_WITH_LIMIT_ASC),
        pool.prepare(UPDATE),
        pool.prepare(DELETE),
    )
    .unwrap();
}

impl MysqlDb {
    pub async fn insert_admin(&self, model: &AdminModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(model.id())
                .bind(model.created_at())
                .bind(model.updated_at())
                .bind(model.email())
                .bind(model.password_hash()),
        )
        .await?;
        Ok(())
    }

    pub async fn upsert_admin(&self, model: &AdminModel) -> Result<()> {
        self.execute(
            sqlx::query(UPSERT)
                .bind(model.id())
                .bind(model.created_at())
                .bind(model.updated_at())
                .bind(model.email())
                .bind(model.password_hash())
                .bind(model.created_at())
                .bind(model.updated_at())
                .bind(model.email())
                .bind(model.password_hash()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_admin(&self, id: &Uuid) -> Result<AdminModel> {
        Ok(self.fetch_one(sqlx::query_as(SELECT).bind(id)).await?)
    }

    pub async fn select_admin_by_email(&self, email: &str) -> Result<AdminModel> {
        Ok(self
            .fetch_one(sqlx::query_as(SELECT_BY_EMAIL).bind(email))
            .await?)
    }

    pub async fn select_many_admins_from_updated_at_and_after_id_with_limit_asc(
        &self,
        updated_at: &DateTime<Utc>,
        id: &Uuid,
        limit: &i32,
    ) -> Result<Vec<AdminModel>> {
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

    pub async fn update_admin(&self, model: &AdminModel) -> Result<()> {
        self.execute(
            sqlx::query(UPDATE)
                .bind(model.updated_at())
                .bind(model.email())
                .bind(model.password_hash())
                .bind(model.id()),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_admin(&self, id: &Uuid) -> Result<()> {
        self.execute(sqlx::query(DELETE).bind(id)).await?;
        Ok(())
    }
}
