use anyhow::Result;
use sqlx::{Executor, Pool, Sqlite};
use uuid::Uuid;

use crate::{db::SqliteDb, model::collection::CollectionModel};

const INSERT: &str = "INSERT INTO \"collections\" (\"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"schema_fields\", \"indexes\", \"auth_columns\") VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"schema_fields\", \"indexes\", \"auth_columns\" FROM \"collections\" WHERE \"id\" = ?";
const SELECT_MANY_BY_PROJECT_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"schema_fields\", \"indexes\", \"auth_columns\" FROM \"collections\" WHERE \"project_id\" = ?";
const UPDATE: &str = "UPDATE \"collections\" SET \"updated_at\" = ?, \"name\" = ?, \"schema_fields\" = ?, \"indexes\" = ?, \"auth_columns\" = ? WHERE \"id\" = ?";
const DELETE: &str = "DELETE FROM \"collections\" WHERE \"id\" = ?";

pub async fn init(pool: &Pool<Sqlite>) {
    hb_log::info(Some("🔧"), "SQLite: Setting up collections table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"collections\" (\"id\" blob, \"created_at\" timestamp, \"updated_at\" timestamp, \"project_id\" blob, \"name\" text, \"schema_fields\" blob, \"indexes\" blob, \"auth_columns\" blob, PRIMARY KEY (\"id\"))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(SELECT_MANY_BY_PROJECT_ID).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}

impl SqliteDb {
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
