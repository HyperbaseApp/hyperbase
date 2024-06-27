use anyhow::Result;
use sqlx::{Executor, MySql, Pool};
use uuid::Uuid;

use crate::{db::MysqlDb, model::collection::CollectionModel};

const INSERT: &str = "INSERT INTO `collections` (`id`, `created_at`, `updated_at`, `project_id`, `name`, `schema_fields`, `opt_auth_column_id`, `opt_ttl`) VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT `id`, `created_at`, `updated_at`, `project_id`, `name`, `schema_fields`, `opt_auth_column_id`, `opt_ttl` FROM `collections` WHERE `id` = ?";
const SELECT_MANY_BY_PROJECT_ID: &str = "SELECT `id`, `created_at`, `updated_at`, `project_id`, `name`, `schema_fields`, `opt_auth_column_id`, `opt_ttl` FROM `collections` WHERE `project_id` = ? ORDER BY `id` DESC";
const UPDATE: &str = "UPDATE `collections` SET `updated_at` = ?, `name` = ?, `schema_fields` = ?, `opt_auth_column_id` = ?, `opt_ttl` = ? WHERE `id` = ?";
const DELETE: &str = "DELETE FROM `collections` WHERE `id` = ?";

pub async fn init(pool: &Pool<MySql>) {
    hb_log::info(Some("🔧"), "[MySQL] Setting up collections table");

    pool.execute("CREATE TABLE IF NOT EXISTS `collections` (`id` binary(16), `created_at` timestamp(6), `updated_at` timestamp(6), `project_id` binary(16), `name` text, `schema_fields` json, `opt_auth_column_id` boolean, `opt_ttl` bigint, PRIMARY KEY (`id`))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT),
        pool.prepare(SELECT_MANY_BY_PROJECT_ID),
        pool.prepare(UPDATE),
        pool.prepare(DELETE),
    )
    .unwrap();
}

impl MysqlDb {
    pub async fn insert_collection(&self, value: &CollectionModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(value.id())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.project_id())
                .bind(value.name())
                .bind(value.schema_fields())
                .bind(value.opt_auth_column_id())
                .bind(value.opt_ttl()),
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
                .bind(value.opt_auth_column_id())
                .bind(value.opt_ttl())
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
