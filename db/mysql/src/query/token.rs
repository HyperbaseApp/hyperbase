use sqlx::{Executor, MySql, Pool};

pub const INSERT: &str = "INSERT INTO `tokens` (`id`, `created_at`, `updated_at`, `admin_id`, `token`, `rules`, `expired_at`) VALUES (?, ?, ?, ?, ?, ?, ?)";
pub const SELECT: &str = "SELECT `id`, `created_at`, `updated_at`, `admin_id`, `token`, `rules`, `expired_at` FROM `tokens` WHERE `id` = ?";
pub const SELECT_MANY_BY_ADMIN_ID: &str = "SELECT `id`, `created_at`, `updated_at`, `admin_id`, `token`, `rules`, `expired_at` FROM `tokens` WHERE `admin_id` = ?";
pub const SELECT_BY_TOKEN: &str = "SELECT `id`, `created_at`, `updated_at`, `admin_id`, `token`, `rules`, `expired_at` FROM `tokens` WHERE `token` = ?";
pub const UPDATE: &str = "UPDATE `tokens` SET `updated_at` = ?, `rules` = ?, `expired_at` = ? WHERE `id` = ?";
pub const DELETE: &str = "DELETE FROM `tokens` WHERE `id` = ?";

pub async fn init(pool: &Pool<MySql>) {
    hb_log::info(Some("🔧"), "MySQL: Setting up tokens table");

    pool.execute("CREATE TABLE IF NOT EXISTS `tokens` (`id` varchar(36), `created_at` timestamp, `updated_at` timestamp, `admin_id` varchar(36), `token` text, `rules` json, `expired_at` timestamp, PRIMARY KEY (`id`))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(SELECT_MANY_BY_ADMIN_ID).await.unwrap();
    pool.prepare(SELECT_BY_TOKEN).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}