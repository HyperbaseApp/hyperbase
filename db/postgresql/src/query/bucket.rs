use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Executor, Pool, Postgres};
use uuid::Uuid;

use crate::{db::PostgresDb, model::bucket::BucketModel};

const INSERT: &str = "INSERT INTO \"buckets\" (\"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"path\", \"opt_ttl\") VALUES ($1, $2, $3, $4, $5, $6, $7)";
const UPSERT: &str = "INSERT INTO \"buckets\" (\"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"path\", \"opt_ttl\") VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (\"id\") DO UPDATE SET \"created_at\" = $2, \"updated_at\" = $3, \"project_id\" = $4, \"name\" = $5, \"path\" = $6, \"opt_ttl\" = $7";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"path\", \"opt_ttl\" FROM \"buckets\" WHERE \"id\" = $1";
const SELECT_MANY_BY_PROJECT_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"path\", \"opt_ttl\" FROM \"buckets\" WHERE \"project_id\" = $1 ORDER BY \"id\" DESC";
const SELECT_MANY_FROM_UPDATED_AT_AND_AFTER_ID_WITH_LIMIT_ASC: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"path\", \"opt_ttl\" FROM \"buckets\" WHERE \"updated_at\" > $1 OR (\"updated_at\" = $1 AND \"id\" > $2) ORDER BY \"updated_at\" ASC, \"id\" ASC LIMIT $3";
const UPDATE: &str = "UPDATE \"buckets\" SET \"updated_at\" = $1, \"name\" = $2, \"opt_ttl\" = $3 WHERE \"id\" = $4";
const DELETE: &str = "DELETE FROM \"buckets\" WHERE \"id\" = $1";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "[PostgreSQL] Setting up buckets table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"buckets\" (\"id\" uuid, \"created_at\" timestamptz, \"updated_at\" timestamptz, \"project_id\" uuid, \"name\" text, \"path\" text, \"opt_ttl\" bigint, PRIMARY KEY (\"id\"))").await.unwrap();

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

impl PostgresDb {
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
