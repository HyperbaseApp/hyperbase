use anyhow::Result;
use sqlx::{Executor, Pool, Sqlite};
use uuid::Uuid;

use crate::{db::SqliteDb, model::admin::AdminModel};

const INSERT: &str = "INSERT INTO \"admins\" (\"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\") VALUES (?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\" FROM \"admins\" WHERE \"id\" = ?";
const SELECT_BY_EMAIL: &str= "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\" FROM \"admins\" WHERE \"email\" = ?";
const UPDATE: &str = "UPDATE \"admins\" SET \"updated_at\" = ?, \"email\" = ?, \"password_hash\" = ? WHERE \"id\" = ?";
const DELETE: &str = "DELETE FROM \"admins\" WHERE \"id\" = ?";

pub async fn init(pool: &Pool<Sqlite>) {
    hb_log::info(Some("🔧"), "[SQLite] Setting up admins table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"admins\" (\"id\" blob, \"created_at\" timestamp, \"updated_at\" timestamp, \"email\" text, \"password_hash\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT),
        pool.prepare(SELECT_BY_EMAIL),
        pool.prepare(UPDATE),
        pool.prepare(DELETE),
    )
    .unwrap();
}

impl SqliteDb {
    pub async fn insert_admin(&self, value: &AdminModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(value.id())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.email())
                .bind(value.password_hash()),
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

    pub async fn update_admin(&self, value: &AdminModel) -> Result<()> {
        self.execute(
            sqlx::query(UPDATE)
                .bind(value.updated_at())
                .bind(value.email())
                .bind(value.password_hash())
                .bind(value.id()),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_admin(&self, id: &Uuid) -> Result<()> {
        self.execute(sqlx::query(DELETE).bind(id)).await?;
        Ok(())
    }
}
