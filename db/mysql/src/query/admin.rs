use anyhow::Result;
use sqlx::{Executor, MySql, Pool};
use uuid::Uuid;

use crate::{db::MysqlDb, model::admin::AdminModel};

const INSERT: &str = "INSERT INTO `admins` (`id`, `created_at`, `updated_at`, `email`, `password_hash`) VALUES (?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT `id`, `created_at`, `updated_at`, `email`, `password_hash` FROM `admins` WHERE `id` = ?";
const SELECT_BY_EMAIL: &str= "SELECT `id`, `created_at`, `updated_at`, `email`, `password_hash` FROM `admins` WHERE `email` = ?";
const SELECT_ALL: &str = "SELECT `id`, `created_at`, `updated_at`, `email`, `password_hash` FROM `admins`";
const UPDATE: &str = "UPDATE `admins` SET `updated_at` = ?, `email` = ?, `password_hash` = ? WHERE `id` = ?";
const DELETE: &str = "DELETE FROM `admins` WHERE `id` = ?";

pub async fn init(pool: &Pool<MySql>) {
    hb_log::info(Some("🔧"), "[MySQL] Setting up admins table");

    pool.execute("CREATE TABLE IF NOT EXISTS `admins` (`id` binary(16), `created_at` timestamp, `updated_at` timestamp, `email` text, `password_hash` text, PRIMARY KEY (`id`))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT),
        pool.prepare(SELECT_BY_EMAIL),
        pool.prepare(UPDATE),
        pool.prepare(DELETE),
    )
    .unwrap();
}

impl MysqlDb {
    pub async fn insert_admin(&self, model: &AdminModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(model.id())
                .bind(model.created_at())
                .bind(model.updated_at())
                .bind(model.email())
                .bind(model.password_hash()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_admin(&self, id: &Uuid) -> Result<AdminModel> {
        Ok(self.fetch_one(sqlx::query_as(SELECT).bind(id)).await?)
    }

    pub async fn select_admin_by_email(&self, email: &str) -> Result<AdminModel> {
        Ok(self
            .fetch_one(sqlx::query_as(SELECT_BY_EMAIL).bind(email))
            .await?)
    }

    pub async fn select_many_admins_after_id_with_limit(
        &self,
        after_id: &Option<Uuid>,
        limit: &i32,
    ) -> Result<Vec<AdminModel>> {
        let mut query = SELECT_ALL.to_owned();

        if after_id.is_some() {
            query += " WHERE `id` > ?";
        }

        query += " ORDER BY `id` ASC LIMIT ?";

        let mut query = sqlx::query_as(&query);
        if let Some(after_id) = after_id {
            query = query.bind(after_id);
        }

        query = query.bind(limit);

        Ok(self.fetch_all(query).await?)
    }

    pub async fn update_admin(&self, model: &AdminModel) -> Result<()> {
        self.execute(
            sqlx::query(UPDATE)
                .bind(model.updated_at())
                .bind(model.email())
                .bind(model.password_hash())
                .bind(model.id()),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_admin(&self, id: &Uuid) -> Result<()> {
        self.execute(sqlx::query(DELETE).bind(id)).await?;
        Ok(())
    }
}
