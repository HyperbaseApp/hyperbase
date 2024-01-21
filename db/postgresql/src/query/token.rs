use anyhow::Result;
use sqlx::{Executor, Pool, Postgres};
use uuid::Uuid;

use crate::{db::PostgresDb, model::token::TokenModel};

const INSERT: &str = "INSERT INTO \"tokens\" (\"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"bucket_rules\", \"collection_rules\", \"expired_at\") VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"bucket_rules\", \"collection_rules\", \"expired_at\" FROM \"tokens\" WHERE \"id\" = $1";
const SELECT_MANY_BY_ADMIN_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"bucket_rules\", \"collection_rules\", \"expired_at\" FROM \"tokens\" WHERE \"admin_id\" = $1";
const SELECT_BY_TOKEN: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"bucket_rules\", \"collection_rules\", \"expired_at\" FROM \"tokens\" WHERE \"token\" = $1";
const UPDATE: &str = "UPDATE \"tokens\" SET \"updated_at\" = $1, \"bucket_rules\" = $2, \"collection_rules\" = $3, \"expired_at\" = $4 WHERE \"id\" = $5";
const DELETE: &str = "DELETE FROM \"tokens\" WHERE \"id\" = $1";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "PostgreSQL: Setting up tokens table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"tokens\" (\"id\" uuid, \"created_at\" timestamptz, \"updated_at\" timestamptz, \"admin_id\" uuid, \"token\" text, \"bucket_rules\" jsonb, \"collection_rules\" jsonb, \"expired_at\" timestamptz, PRIMARY KEY (\"id\"))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(SELECT_MANY_BY_ADMIN_ID).await.unwrap();
    pool.prepare(SELECT_BY_TOKEN).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}

impl PostgresDb {
    pub async fn insert_token(&self, value: &TokenModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(value.id())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.admin_id())
                .bind(value.token())
                .bind(value.collection_rules())
                .bind(value.expired_at()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_token(&self, id: &Uuid) -> Result<TokenModel> {
        Ok(self.fetch_one(sqlx::query_as(SELECT).bind(id)).await?)
    }

    pub async fn select_token_by_token(&self, token: &str) -> Result<TokenModel> {
        Ok(self
            .fetch_one(sqlx::query_as(SELECT_BY_TOKEN).bind(token))
            .await?)
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
