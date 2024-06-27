use anyhow::Result;
use sqlx::{Executor, MySql, Pool};
use uuid::Uuid;

use crate::{db::MysqlDb, model::project::ProjectModel};

const INSERT: &str = "INSERT INTO `projects` (`id`, `created_at`, `updated_at`, `admin_id`, `name`) VALUES (?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT `id`, `created_at`, `updated_at`, `admin_id`, `name` FROM `projects` WHERE `id` = ?";
const SELECT_MANY_BY_ADMIN_ID:  &str = "SELECT `id`, `created_at`, `updated_at`, `admin_id`, `name` FROM `projects` WHERE `admin_id` = ? ORDER BY `id` DESC";
const UPDATE: &str = "UPDATE `projects` SET `updated_at` = ?, `admin_id` = ?, `name` = ? WHERE `id` = ?";
const DELETE: &str = "DELETE FROM `projects` WHERE `id` = ?";

pub async fn init(pool: &Pool<MySql>) {
    hb_log::info(Some("🔧"), "[MySQL] Setting up projects table");

    pool.execute("CREATE TABLE IF NOT EXISTS `projects` (`id` binary(16), `created_at` timestamp(6), `updated_at` timestamp(6), `admin_id` binary(16), `name` text, PRIMARY KEY (`id`))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT),
        pool.prepare(SELECT_MANY_BY_ADMIN_ID),
        pool.prepare(UPDATE),
        pool.prepare(DELETE),
    )
    .unwrap();
}

impl MysqlDb {
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
