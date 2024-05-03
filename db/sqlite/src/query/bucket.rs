use anyhow::Result;
use sqlx::{Executor, Pool, Sqlite};
use uuid::Uuid;

use crate::{db::SqliteDb, model::bucket::BucketModel};

const INSERT: &str = "INSERT INTO \"buckets\" (\"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"path\", \"opt_ttl\") VALUES (?, ?, ?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"path\", \"opt_ttl\" FROM \"buckets\" WHERE \"id\" = ?";
const SELECT_MANY_BY_PROJECT_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"path\", \"opt_ttl\" FROM \"buckets\" WHERE \"project_id\" = ? ORDER BY \"id\" DESC";
const SELECT_ALL: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"path\", \"opt_ttl\" FROM \"buckets\"";
const UPDATE: &str = "UPDATE \"buckets\" SET \"updated_at\" = ?, \"name\" = ?, \"opt_ttl\" = ? WHERE \"id\" = ?";
const DELETE: &str = "DELETE FROM \"buckets\" WHERE \"id\" = ?";

pub async fn init(pool: &Pool<Sqlite>) {
    hb_log::info(Some("🔧"), "[SQLite] Setting up buckets table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"buckets\" (\"id\" blob, \"created_at\" timestamp, \"updated_at\" timestamp, \"project_id\" blob, \"name\" text, \"path\" text, \"opt_ttl\" bigint, PRIMARY KEY (\"id\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT),
        pool.prepare(SELECT_MANY_BY_PROJECT_ID),
        pool.prepare(UPDATE),
        pool.prepare(DELETE),
    )
    .unwrap();
}

impl SqliteDb {
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

    pub async fn select_many_buckets_after_id_with_limit(
        &self,
        after_id: &Option<Uuid>,
        limit: &i32,
    ) -> Result<Vec<BucketModel>> {
        let mut query = SELECT_ALL.to_owned();

        if after_id.is_some() {
            query += " WHERE \"id\" > ?";
        }

        query += " ORDER BY \"id\" ASC LIMIT ?";

        let mut query = sqlx::query_as(&query);
        if let Some(after_id) = after_id {
            query = query.bind(after_id);
        }

        query = query.bind(limit);

        Ok(self.fetch_all(query).await?)
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
