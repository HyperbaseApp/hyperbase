use sqlx::{Executor, MySql, Pool};

pub const INSERT: &str = "INSERT INTO `collections` (`id`, `created_at`, `updated_at`, `project_id`, `name`, `schema_fields`, `indexes`) VALUES (?, ?, ?, ?, ?, ?, ?)";
pub const SELECT: &str = "SELECT `id`, `created_at`, `updated_at`, `project_id`, `name`, `schema_fields`, `indexes` FROM `collections` WHERE `id` = ?";
pub const SELECT_MANY_BY_PROJECT_ID: &str = "SELECT `id`, `created_at`, `updated_at`, `project_id`, `name`, `schema_fields`, `indexes` FROM `collections` WHERE `project_id` = ?";
pub const UPDATE: &str = "UPDATE `collections` SET `updated_at` = ?, `name` = ?, `schema_fields` = ?, `indexes` = ? WHERE `id` = ?";
pub const DELETE: &str = "DELETE FROM `collections` WHERE `id` = ?";

pub async fn init(pool: &Pool<MySql>) {
    hb_log::info(Some("🔧"), "MySQL: Setting up collections table");

    pool.execute("CREATE TABLE IF NOT EXISTS `collections` (`id` binary(16)	, `created_at` timestamp, `updated_at` timestamp, `project_id` binary(16), `name` text, `schema_fields` json, `indexes` json, PRIMARY KEY (`id`))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(SELECT_MANY_BY_PROJECT_ID).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}
