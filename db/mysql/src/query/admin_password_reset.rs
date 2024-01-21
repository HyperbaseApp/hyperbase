use anyhow::Result;
use sqlx::{
    types::chrono::{DateTime, Utc},
    Executor, MySql, Pool,
};
use uuid::Uuid;

use crate::{db::MysqlDb, model::admin_password_reset::AdminPasswordResetModel};

const INSERT: &str = "INSERT INTO `admin_password_resets` (`id`, `created_at`, `updated_at`, `admin_id`, `code`) VALUES (?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT `id`, `created_at`, `updated_at`, `admin_id`, `code` FROM `admin_password_resets` WHERE `id` = ? AND `updated_at` >= ?";
const UPDATE: &str = "UPDATE `admin_password_resets` SET `updated_at` = ?, `code` = ? WHERE `id` = ? AND `updated_at` >= ?";
const DELETE: &str = "DELETE FROM `admin_password_resets` WHERE `id` = ?";
const DELETE_EXPIRE: &str = "DELETE FROM `admin_password_resets` WHERE `updated_at` < ?";

pub async fn init(pool: &Pool<MySql>) {
    hb_log::info(Some("🔧"), "MySQL: Setting up admin_password_resets table");

    pool.execute("CREATE TABLE IF NOT EXISTS `admin_password_resets` (`id` binary(16)	, `created_at` timestamp, `updated_at` timestamp, `admin_id` binary(16)	, `code` text, PRIMARY KEY (`id`))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}

impl MysqlDb {
    pub async fn insert_admin_password_reset(&self, value: &AdminPasswordResetModel) -> Result<()> {
        let _ = self.delete_expired_admin_password_reset().await;
        self.execute(
            sqlx::query(INSERT)
                .bind(value.id())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.admin_id())
                .bind(value.code()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_admin_password_reset(&self, id: &Uuid) -> Result<AdminPasswordResetModel> {
        let _ = self.delete_expired_admin_password_reset().await;
        Ok(self
            .fetch_one(sqlx::query_as(SELECT).bind(id).bind(&{
                let now = Utc::now();
                DateTime::from_timestamp(
                    now.timestamp() - self.table_reset_password_ttl(),
                    now.timestamp_subsec_nanos(),
                )
                .unwrap()
            }))
            .await?)
    }

    pub async fn update_admin_password_reset(&self, value: &AdminPasswordResetModel) -> Result<()> {
        let _ = self.delete_expired_admin_password_reset().await;
        self.execute(
            sqlx::query(UPDATE)
                .bind(value.updated_at())
                .bind(value.code())
                .bind(value.id())
                .bind(&{
                    let now = Utc::now();
                    DateTime::from_timestamp(
                        now.timestamp() - self.table_reset_password_ttl(),
                        now.timestamp_subsec_nanos(),
                    )
                    .unwrap()
                }),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_admin_password_reset(&self, id: &Uuid) -> Result<()> {
        let _ = self.delete_expired_admin_password_reset().await;
        self.execute(sqlx::query(DELETE).bind(id)).await?;
        Ok(())
    }

    async fn delete_expired_admin_password_reset(&self) -> Result<()> {
        self.execute(sqlx::query(DELETE_EXPIRE).bind(self.table_reset_password_ttl()))
            .await?;
        Ok(())
    }
}
