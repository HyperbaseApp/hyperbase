use anyhow::Result;
use sqlx::{
    types::chrono::{DateTime, Utc},
    Executor, Pool, Sqlite,
};
use uuid::Uuid;

use crate::{db::SqliteDb, model::registration::RegistrationModel};

const INSERT: &str = "INSERT INTO \"registrations\" (\"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\", \"code\") VALUES (?, ?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\", \"code\" FROM \"registrations\" WHERE \"id\" = ? AND \"updated_at\" >= ?";
const SELECT_BY_EMAIL: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\", \"code\" FROM \"registrations\" WHERE \"email\" = ? AND \"updated_at\" >= ?";
const UPDATE: &str = "UPDATE \"registrations\" SET \"updated_at\" = ?, \"code\" = ? WHERE \"id\" = ?";
const DELETE: &str = "DELETE FROM \"registrations\" WHERE \"id\" = ?";
const DELETE_EXPIRE: &str = "DELETE FROM \"registrations\" WHERE \"updated_at\" < ?";

pub async fn init(pool: &Pool<Sqlite>) {
    hb_log::info(Some("🔧"), "SQLite: Setting up registrations table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"registrations\" (\"id\" blob, \"created_at\" timestamp, \"updated_at\" timestamp, \"email\" text, \"password_hash\" text, \"code\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}

impl SqliteDb {
    pub async fn insert_registration(&self, value: &RegistrationModel) -> Result<()> {
        let _ = self.delete_expired_registration().await;
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
        let _ = self.delete_expired_registration().await;
        Ok(self
            .fetch_one(sqlx::query_as(SELECT).bind(id).bind(&{
                let now = Utc::now();
                DateTime::from_timestamp(
                    now.timestamp() - self.table_registration_ttl(),
                    now.timestamp_subsec_nanos(),
                )
                .unwrap()
            }))
            .await?)
    }

    pub async fn select_registration_by_email(&self, email: &str) -> Result<RegistrationModel> {
        let _ = self.delete_expired_registration().await;
        Ok(self
            .fetch_one(sqlx::query_as(SELECT_BY_EMAIL).bind(email).bind(&{
                let now = Utc::now();
                DateTime::from_timestamp(
                    now.timestamp() - self.table_registration_ttl(),
                    now.timestamp_subsec_nanos(),
                )
                .unwrap()
            }))
            .await?)
    }

    pub async fn update_registration(&self, value: &RegistrationModel) -> Result<()> {
        let _ = self.delete_expired_registration().await;
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
        let _ = self.delete_expired_registration().await;
        self.execute(sqlx::query(DELETE).bind(id)).await?;
        Ok(())
    }

    async fn delete_expired_registration(&self) -> Result<()> {
        self.fetch_one(sqlx::query_as(DELETE_EXPIRE).bind(self.table_registration_ttl()))
            .await?;
        Ok(())
    }
}
