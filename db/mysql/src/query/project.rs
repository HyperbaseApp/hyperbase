use sqlx::{Executor, MySql, Pool};

pub const INSERT: &str = "INSERT INTO `projects` (`id`, `created_at`, `updated_at`, `admin_id`, `name`) VALUES (?, ?, ?, ?, ?)";
pub const SELECT: &str = "SELECT `id`, `created_at`, `updated_at`, `admin_id`, `name` FROM `projects` WHERE `id` = ?";
pub const SELECT_MANY_BY_ADMIN_ID :  &str = "SELECT `id`, `created_at`, `updated_at`, `admin_id`, `name` FROM `projects` WHERE `admin_id` = ?";
pub const UPDATE: &str = "UPDATE `projects` SET `updated_at` = ?, `name` = ? WHERE `id` = ?";
pub const DELETE: &str = "DELETE FROM `projects` WHERE `id` = ?";

pub async fn init(pool: &Pool<MySql>) {
    hb_log::info(Some("🔧"), "MySQL: Setting up projects table");

    pool.execute("CREATE TABLE IF NOT EXISTS `projects` (`id` binary(16)	, `created_at` timestamp, `updated_at` timestamp, `admin_id` binary(16)	, `name` text, PRIMARY KEY (`id`))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(SELECT_MANY_BY_ADMIN_ID).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}
