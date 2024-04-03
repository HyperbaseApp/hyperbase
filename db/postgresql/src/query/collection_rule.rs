use anyhow::Result;
use sqlx::{Executor, Pool, Postgres};
use uuid::Uuid;

use crate::{db::PostgresDb, model::collection_rule::CollectionRuleModel};

const INSERT: &str = "INSERT INTO \"collection_rules\" (\"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"collection_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\") VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"collection_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\" FROM \"collection_rules\" WHERE \"id\" = $1";
const SELECT_BY_TOKEN_ID_AND_COLLECTION_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"collection_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\" FROM \"collection_rules\" WHERE \"token_id\" = $1 AND \"collection_id\" = $2";
const SELECT_MANY_BY_TOKEN_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"collection_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\" FROM \"collection_rules\" WHERE \"token_id\" = $1 ORDER BY \"created_at\" DESC";
const UPDATE: &str = "UPDATE \"collection_rules\" SET \"updated_at\" = $1, \"find_one\" = $2, \"find_many\" = $3, \"insert_one\" = $4, \"update_one\" = $5, \"delete_one\" = $6 WHERE \"id\" = $7";
const DELETE: &str = "DELETE FROM \"collection_rules\" WHERE \"id\" = $1";
const DELETE_MANY_BY_TOKEN_ID: &str = "DELETE FROM \"collection_rules\" WHERE \"token_id\" = $1";
const DELETE_MANY_BY_COLLECTION_ID: &str = "DELETE FROM \"collection_rules\" WHERE \"collection_id\" = $1";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "PostgreSQL: Setting up collection_rules table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"collection_rules\" (\"id\" uuid, \"created_at\" timestamptz, \"updated_at\" timestamptz, \"project_id\" uuid, \"token_id\" uuid, \"collection_id\" uuid, \"find_one\" text, \"find_many\" text, \"insert_one\" boolean, \"update_one\" text, \"delete_one\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(SELECT_BY_TOKEN_ID_AND_COLLECTION_ID)
        .await
        .unwrap();
    pool.prepare(SELECT_MANY_BY_TOKEN_ID).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
    pool.prepare(DELETE_MANY_BY_TOKEN_ID).await.unwrap();
    pool.prepare(DELETE_MANY_BY_COLLECTION_ID).await.unwrap();
}

impl PostgresDb {
    pub async fn insert_collection_rule(&self, value: &CollectionRuleModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(value.id())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.project_id())
                .bind(value.token_id())
                .bind(value.collection_id())
                .bind(value.find_one())
                .bind(value.find_many())
                .bind(value.insert_one())
                .bind(value.update_one())
                .bind(value.delete_one()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_collection_rule(&self, id: &Uuid) -> Result<CollectionRuleModel> {
        Ok(self.fetch_one(sqlx::query_as(SELECT).bind(id)).await?)
    }

    pub async fn select_collection_rule_by_token_id_and_collection_id(
        &self,
        token_id: &Uuid,
        collection_id: &Uuid,
    ) -> Result<CollectionRuleModel> {
        Ok(self
            .fetch_one(
                sqlx::query_as(SELECT_BY_TOKEN_ID_AND_COLLECTION_ID)
                    .bind(token_id)
                    .bind(collection_id),
            )
            .await?)
    }

    pub async fn select_many_collection_rules_by_token_id(
        &self,
        token_id: &Uuid,
    ) -> Result<Vec<CollectionRuleModel>> {
        Ok(self
            .fetch_all(sqlx::query_as(SELECT_MANY_BY_TOKEN_ID).bind(token_id))
            .await?)
    }

    pub async fn update_collection_rule(&self, value: &CollectionRuleModel) -> Result<()> {
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

    pub async fn delete_collection_rule(&self, id: &Uuid) -> Result<()> {
        self.execute(sqlx::query(DELETE).bind(id)).await?;
        Ok(())
    }

    pub async fn delete_many_collection_rules_by_token_id(&self, token_id: &Uuid) -> Result<()> {
        self.execute(sqlx::query(DELETE_MANY_BY_TOKEN_ID).bind(token_id))
            .await?;
        Ok(())
    }

    pub async fn delete_many_collection_rules_by_collection_id(
        &self,
        collection_id: &Uuid,
    ) -> Result<()> {
        self.execute(sqlx::query(DELETE_MANY_BY_COLLECTION_ID).bind(collection_id))
            .await?;
        Ok(())
    }
}
