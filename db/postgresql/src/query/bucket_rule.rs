use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Executor, Pool, Postgres};
use uuid::Uuid;

use crate::{db::PostgresDb, model::bucket_rule::BucketRuleModel};

const INSERT: &str = "INSERT INTO \"bucket_rules\" (\"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"bucket_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\") VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)";
const UPSERT: &str = "INSERT INTO \"bucket_rules\" (\"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"bucket_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\") VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) ON CONFLICT (\"id\") DO UPDATE SET \"created_at\" = $2, \"updated_at\" = $3, \"project_id\" = $4, \"token_id\" = $5, \"bucket_id\" = $6, \"find_one\" = $7, \"find_many\" = $8, \"insert_one\" = $9, \"update_one\" = $10, \"delete_one\" = $11";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"bucket_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\" FROM \"bucket_rules\" WHERE \"id\" = $1";
const SELECT_BY_TOKEN_ID_AND_BUCKET_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"bucket_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\" FROM \"bucket_rules\" WHERE \"token_id\" = $1 AND \"bucket_id\" = $2";
const SELECT_MANY_BY_TOKEN_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"bucket_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\" FROM \"bucket_rules\" WHERE \"token_id\" = $1 ORDER BY \"id\" DESC";
const SELECT_MANY_FROM_UPDATED_AT_AND_AFTER_ID_WITH_LIMIT_ASC: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"bucket_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\" FROM \"bucket_rules\" WHERE \"updated_at\" > $1 OR (\"updated_at\" = $1 AND \"id\" > $2) ORDER BY \"updated_at\" ASC, \"id\" ASC LIMIT $3";
const UPDATE: &str = "UPDATE \"bucket_rules\" SET \"updated_at\" = $1, \"find_one\" = $2, \"find_many\" = $3, \"insert_one\" = $4, \"update_one\" = $5, \"delete_one\" = $6 WHERE \"id\" = $7";
const DELETE: &str = "DELETE FROM \"bucket_rules\" WHERE \"id\" = $1";
const DELETE_MANY_BY_TOKEN_ID: &str = "DELETE FROM \"bucket_rules\" WHERE \"token_id\" = $1";
const DELETE_MANY_BY_BUCKET_ID: &str = "DELETE FROM \"bucket_rules\" WHERE \"bucket_id\" = $1";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "[PostgreSQL] Setting up bucket_rules table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"bucket_rules\" (\"id\" uuid, \"created_at\" timestamptz, \"updated_at\" timestamptz, \"project_id\" uuid, \"token_id\" uuid, \"bucket_id\" uuid, \"find_one\" text, \"find_many\" text, \"insert_one\" boolean, \"update_one\" text, \"delete_one\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(UPSERT),
        pool.prepare(SELECT),
        pool.prepare(SELECT_BY_TOKEN_ID_AND_BUCKET_ID),
        pool.prepare(SELECT_MANY_BY_TOKEN_ID),
        pool.prepare(SELECT_MANY_FROM_UPDATED_AT_AND_AFTER_ID_WITH_LIMIT_ASC),
        pool.prepare(UPDATE),
        pool.prepare(DELETE),
        pool.prepare(DELETE_MANY_BY_TOKEN_ID),
        pool.prepare(DELETE_MANY_BY_BUCKET_ID),
    )
    .unwrap();
}

impl PostgresDb {
    pub async fn insert_bucket_rule(&self, value: &BucketRuleModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(value.id())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.project_id())
                .bind(value.token_id())
                .bind(value.bucket_id())
                .bind(value.find_one())
                .bind(value.find_many())
                .bind(value.insert_one())
                .bind(value.update_one())
                .bind(value.delete_one()),
        )
        .await?;
        Ok(())
    }

    pub async fn upsert_bucket_rule(&self, value: &BucketRuleModel) -> Result<()> {
        self.execute(
            sqlx::query(UPSERT)
                .bind(value.id())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.project_id())
                .bind(value.token_id())
                .bind(value.bucket_id())
                .bind(value.find_one())
                .bind(value.find_many())
                .bind(value.insert_one())
                .bind(value.update_one())
                .bind(value.delete_one()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_bucket_rule(&self, id: &Uuid) -> Result<BucketRuleModel> {
        Ok(self.fetch_one(sqlx::query_as(SELECT).bind(id)).await?)
    }

    pub async fn select_bucket_rule_by_token_id_and_bucket_id(
        &self,
        token_id: &Uuid,
        bucket_id: &Uuid,
    ) -> Result<BucketRuleModel> {
        Ok(self
            .fetch_one(
                sqlx::query_as(SELECT_BY_TOKEN_ID_AND_BUCKET_ID)
                    .bind(token_id)
                    .bind(bucket_id),
            )
            .await?)
    }

    pub async fn select_many_bucket_rules_by_token_id(
        &self,
        token_id: &Uuid,
    ) -> Result<Vec<BucketRuleModel>> {
        Ok(self
            .fetch_all(sqlx::query_as(SELECT_MANY_BY_TOKEN_ID).bind(token_id))
            .await?)
    }

    pub async fn select_many_bucket_rules_from_updated_at_and_after_id_with_limit_asc(
        &self,
        updated_at: &DateTime<Utc>,
        id: &Uuid,
        limit: &i32,
    ) -> Result<Vec<BucketRuleModel>> {
        Ok(self
            .fetch_all(
                sqlx::query_as(SELECT_MANY_FROM_UPDATED_AT_AND_AFTER_ID_WITH_LIMIT_ASC)
                    .bind(updated_at)
                    .bind(id)
                    .bind(limit),
            )
            .await?)
    }

    pub async fn update_bucket_rule(&self, value: &BucketRuleModel) -> Result<()> {
        self.execute(
            sqlx::query(UPDATE)
                .bind(value.updated_at())
                .bind(value.find_one())
                .bind(value.find_many())
                .bind(value.insert_one())
                .bind(value.update_one())
                .bind(value.delete_one())
                .bind(value.id()),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_bucket_rule(&self, id: &Uuid) -> Result<()> {
        self.execute(sqlx::query(DELETE).bind(id)).await?;
        Ok(())
    }

    pub async fn delete_many_bucket_rules_by_token_id(&self, token_id: &Uuid) -> Result<()> {
        self.execute(sqlx::query(DELETE_MANY_BY_TOKEN_ID).bind(token_id))
            .await?;
        Ok(())
    }

    pub async fn delete_many_bucket_rules_by_bucket_id(&self, bucket_id: &Uuid) -> Result<()> {
        self.execute(sqlx::query(DELETE_MANY_BY_BUCKET_ID).bind(bucket_id))
            .await?;
        Ok(())
    }
}
