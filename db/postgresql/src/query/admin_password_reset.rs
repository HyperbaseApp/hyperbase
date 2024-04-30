use anyhow::{Error, Result};
use chrono::{Duration, Utc};
use sqlx::{types::chrono::DateTime, Executor, Pool, Postgres};
use uuid::Uuid;

use crate::{db::PostgresDb, model::admin_password_reset::AdminPasswordResetModel};

const INSERT: &str = "INSERT INTO \"admin_password_resets\" (\"id\", \"created_at\", \"updated_at\", \"admin_id\", \"code\") VALUES ($1, $2, $3, $4, $5)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"code\" FROM \"admin_password_resets\" WHERE \"id\" = $1 AND \"updated_at\" >= $2";
const UPDATE: &str = "UPDATE \"admin_password_resets\" SET \"updated_at\" = $1, \"code\" = $2 WHERE \"id\" = $3 AND \"updated_at\" >= $4";
const DELETE: &str = "DELETE FROM \"admin_password_resets\" WHERE \"id\" = $1";
const DELETE_EXPIRE: &str = "DELETE FROM \"admin_password_resets\" WHERE \"updated_at\" < $1";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(
        Some("🔧"),
        "[PostgreSQL] Setting up admin_password_resets table",
    );

    pool.execute("CREATE TABLE IF NOT EXISTS \"admin_password_resets\" (\"id\" uuid, \"created_at\" timestamptz, \"updated_at\" timestamptz, \"admin_id\" uuid, \"code\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT),
        pool.prepare(UPDATE),
        pool.prepare(DELETE),
        pool.prepare(DELETE_EXPIRE),
    )
    .unwrap();
}

impl PostgresDb {
    pub async fn insert_admin_password_reset(&self, value: &AdminPasswordResetModel) -> Result<()> {
        let _ = self.delete_expired_admin_password_resets().await;
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
        let _ = self.delete_expired_admin_password_resets().await;
        Ok(self
            .fetch_one(sqlx::query_as(SELECT).bind(id).bind(&{
                let now = Utc::now();
                DateTime::from_timestamp(
                    now.timestamp() - self.table_reset_password_ttl(),
                    now.timestamp_subsec_nanos(),
                )
                .ok_or_else(|| Error::msg("timestamp is out of range."))?
            }))
            .await?)
    }

    pub async fn update_admin_password_reset(&self, value: &AdminPasswordResetModel) -> Result<()> {
        let _ = self.delete_expired_admin_password_resets().await;
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
                    .ok_or_else(|| Error::msg("timestamp is out of range."))?
                }),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_admin_password_reset(&self, id: &Uuid) -> Result<()> {
        let _ = self.delete_expired_admin_password_resets().await;
        self.execute(sqlx::query(DELETE).bind(id)).await?;
        Ok(())
    }

    async fn delete_expired_admin_password_resets(&self) -> Result<()> {
        self.execute(
            sqlx::query(DELETE_EXPIRE).bind(
                Utc::now()
                    .checked_sub_signed(
                        Duration::try_seconds(*self.table_reset_password_ttl()).ok_or_else(
                            || Error::msg("table_reset_password_ttl is out of range."),
                        )?,
                    )
                    .ok_or_else(|| Error::msg("table_reset_password_ttl is out of range."))?,
            ),
        )
        .await?;
        Ok(())
    }
}
