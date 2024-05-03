use anyhow::Result;
use sqlx::{Executor, Pool, Sqlite};
use uuid::Uuid;

use crate::{db::SqliteDb, model::project::ProjectModel};

const INSERT: &str = "INSERT INTO \"projects\" (\"id\", \"created_at\", \"updated_at\", \"admin_id\", \"name\") VALUES (?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"name\" FROM \"projects\" WHERE \"id\" = ?";
const SELECT_MANY_BY_ADMIN_ID:  &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"name\" FROM \"projects\" WHERE \"admin_id\" = ? ORDER BY \"id\" DESC";
const SELECT_ALL: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"name\" FROM \"projects\"";
const UPDATE: &str = "UPDATE \"projects\" SET \"updated_at\" = ?, \"admin_id\" = ?, \"name\" = ? WHERE \"id\" = ?";
const DELETE: &str = "DELETE FROM \"projects\" WHERE \"id\" = ?";

pub async fn init(pool: &Pool<Sqlite>) {
    hb_log::info(Some("🔧"), "[SQLite] Setting up projects table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"projects\" (\"id\" blob, \"created_at\" timestamp, \"updated_at\" timestamp, \"admin_id\" blob, \"name\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT),
        pool.prepare(SELECT_MANY_BY_ADMIN_ID),
        pool.prepare(UPDATE),
        pool.prepare(DELETE),
    )
    .unwrap();
}

impl SqliteDb {
    pub async fn insert_project(&self, value: &ProjectModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(value.id())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.admin_id())
                .bind(value.name()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_project(&self, id: &Uuid) -> Result<ProjectModel> {
        Ok(self.fetch_one(sqlx::query_as(SELECT).bind(id)).await?)
    }

    pub async fn select_many_projects_by_admin_id(
        &self,
        admin_id: &Uuid,
    ) -> Result<Vec<ProjectModel>> {
        Ok(self
            .fetch_all(sqlx::query_as(SELECT_MANY_BY_ADMIN_ID).bind(admin_id))
            .await?)
    }

    pub async fn select_many_projects_after_id_with_limit(
        &self,
        after_id: &Option<Uuid>,
        limit: &i32,
    ) -> Result<Vec<ProjectModel>> {
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

    pub async fn update_project(&self, value: &ProjectModel) -> Result<()> {
        self.execute(
            sqlx::query(UPDATE)
                .bind(value.updated_at())
                .bind(value.admin_id())
                .bind(value.name())
                .bind(value.id()),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_project(&self, id: &Uuid) -> Result<()> {
        self.execute(sqlx::query(DELETE).bind(id)).await?;
        Ok(())
    }
}
