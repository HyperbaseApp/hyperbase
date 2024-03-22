use anyhow::Result;
use sqlx::{Executor, Pool, Postgres};
use uuid::Uuid;

use crate::{db::PostgresDb, model::collection::CollectionModel};

const INSERT: &str = "INSERT INTO \"collections\" (\"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"schema_fields\", \"indexes\", \"auth_columns\") VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"schema_fields\", \"indexes\", \"auth_columns\" FROM \"collections\" WHERE \"id\" = $1";
const SELECT_MANY_BY_PROJECT_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"schema_fields\", \"indexes\", \"auth_columns\" FROM \"collections\" WHERE \"project_id\" = $1";
const UPDATE: &str = "UPDATE \"collections\" SET \"updated_at\" = $1, \"name\" = $2, \"schema_fields\" = $3, \"indexes\" = $4, \"auth_columns\" = $5 WHERE \"id\" = $6";
const DELETE: &str = "DELETE FROM \"collections\" WHERE \"id\" = $1";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "PostgreSQL: Setting up collections table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"collections\" (\"id\" uuid, \"created_at\" timestamptz, \"updated_at\" timestamptz, \"project_id\" uuid, \"name\" text, \"schema_fields\" jsonb, \"indexes\" jsonb, \"auth_columns\" jsonb, PRIMARY KEY (\"id\"))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(SELECT_MANY_BY_PROJECT_ID).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}

impl PostgresDb {
    pub async fn insert_collection(&self, value: &CollectionModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(value.id())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.project_id())
                .bind(value.name())
                .bind(value.schema_fields())
                .bind(value.indexes())
                .bind(value.auth_columns()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_collection(&self, id: &Uuid) -> Result<CollectionModel> {
        Ok(self.fetch_one(sqlx::query_as(SELECT).bind(id)).await?)
    }

    pub async fn select_many_collections_by_project_id(
        &self,
        project_id: &Uuid,
    ) -> Result<Vec<CollectionModel>> {
        Ok(self
            .fetch_all(sqlx::query_as(SELECT_MANY_BY_PROJECT_ID).bind(project_id))
            .await?)
    }

    pub async fn update_collection(&self, value: &CollectionModel) -> Result<()> {
        self.execute(
            sqlx::query(UPDATE)
                .bind(value.updated_at())
                .bind(value.name())
                .bind(value.schema_fields())
                .bind(value.indexes())
                .bind(value.auth_columns())
                .bind(value.id()),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_collection(&self, id: &Uuid) -> Result<()> {
        self.execute(sqlx::query(DELETE).bind(id)).await?;
        Ok(())
    }
}
