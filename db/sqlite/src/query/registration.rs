use anyhow::{Error, Result};
use chrono::{Duration, Utc};
use sqlx::{types::chrono::DateTime, Executor, Pool, Sqlite};
use uuid::Uuid;

use crate::{db::SqliteDb, model::registration::RegistrationModel};

const INSERT: &str = "INSERT INTO \"registrations\" (\"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\", \"code\") VALUES (?, ?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\", \"code\" FROM \"registrations\" WHERE \"id\" = ? AND \"updated_at\" >= ?";
const SELECT_BY_EMAIL: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\", \"code\" FROM \"registrations\" WHERE \"email\" = ? AND \"updated_at\" >= ?";
const UPDATE: &str = "UPDATE \"registrations\" SET \"updated_at\" = ?, \"code\" = ? WHERE \"id\" = ?";
const DELETE: &str = "DELETE FROM \"registrations\" WHERE \"id\" = ?";
const DELETE_EXPIRE: &str = "DELETE FROM \"registrations\" WHERE \"updated_at\" < ?";

pub async fn init(pool: &Pool<Sqlite>) {
    hb_log::info(Some("🔧"), "[SQLite] Setting up registrations table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"registrations\" (\"id\" blob, \"created_at\" timestamp, \"updated_at\" timestamp, \"email\" text, \"password_hash\" text, \"code\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT),
        pool.prepare(SELECT_BY_EMAIL),
        pool.prepare(UPDATE),
        pool.prepare(DELETE),
        pool.prepare(DELETE_EXPIRE),
    )
    .unwrap();
}

impl SqliteDb {
    pub async fn insert_registration(&self, value: &RegistrationModel) -> Result<()> {
        let _ = self.delete_expired_registrations().await;
        self.execute(
            sqlx::query(INSERT)
                .bind(value.id())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.email())
                .bind(value.password_hash())
                .bind(value.code()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_registration(&self, id: &Uuid) -> Result<RegistrationModel> {
        let _ = self.delete_expired_registrations().await;
        Ok(self
            .fetch_one(sqlx::query_as(SELECT).bind(id).bind(&{
                let now = Utc::now();
                DateTime::from_timestamp(
                    now.timestamp() - self.table_registration_ttl(),
                    now.timestamp_subsec_nanos(),
                )
                .ok_or_else(|| Error::msg("timestamp is out of range."))?
            }))
            .await?)
    }

    pub async fn select_registration_by_email(&self, email: &str) -> Result<RegistrationModel> {
        let _ = self.delete_expired_registrations().await;
        Ok(self
            .fetch_one(sqlx::query_as(SELECT_BY_EMAIL).bind(email).bind(&{
                let now = Utc::now();
                DateTime::from_timestamp(
                    now.timestamp() - self.table_registration_ttl(),
                    now.timestamp_subsec_nanos(),
                )
                .ok_or_else(|| Error::msg("timestamp is out of range."))?
            }))
            .await?)
    }

    pub async fn update_registration(&self, value: &RegistrationModel) -> Result<()> {
        let _ = self.delete_expired_registrations().await;
        self.execute(
            sqlx::query(UPDATE)
                .bind(value.updated_at())
                .bind(value.code())
                .bind(value.id()),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_registration(&self, id: &Uuid) -> Result<()> {
        let _ = self.delete_expired_registrations().await;
        self.execute(sqlx::query(DELETE).bind(id)).await?;
        Ok(())
    }

    async fn delete_expired_registrations(&self) -> Result<()> {
        self.execute(
            sqlx::query(DELETE_EXPIRE).bind(
                Utc::now()
                    .checked_sub_signed(
                        Duration::try_seconds(*self.table_registration_ttl())
                            .ok_or_else(|| Error::msg("table_registration_ttl is out of range."))?,
                    )
                    .ok_or_else(|| Error::msg("table_registration_ttl is out of range."))?,
            ),
        )
        .await?;
        Ok(())
    }
}
