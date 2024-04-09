use anyhow::Result;
use sqlx::{Executor, Pool, Postgres};
use uuid::Uuid;

use crate::{db::PostgresDb, model::project::ProjectModel};

const INSERT: &str = "INSERT INTO \"projects\" (\"id\", \"created_at\", \"updated_at\", \"admin_id\", \"name\") VALUES ($1, $2, $3, $4, $5)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"name\" FROM \"projects\" WHERE \"id\" = $1";
const SELECT_MANY_BY_ADMIN_ID:  &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"name\" FROM \"projects\" WHERE \"admin_id\" = $1 ORDER BY \"id\" DESC";
const UPDATE: &str = "UPDATE \"projects\" SET \"updated_at\" = $1, \"admin_id\" = $2, \"name\" = $3 WHERE \"id\" = $4";
const DELETE: &str = "DELETE FROM \"projects\" WHERE \"id\" = $1";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "PostgreSQL: Setting up projects table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"projects\" (\"id\" uuid, \"created_at\" timestamptz, \"updated_at\" timestamptz, \"admin_id\" uuid, \"name\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT),
        pool.prepare(SELECT_MANY_BY_ADMIN_ID),
        pool.prepare(UPDATE),
        pool.prepare(DELETE),
    )
    .unwrap();
}

impl PostgresDb {
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
