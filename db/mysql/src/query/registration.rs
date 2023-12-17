use sqlx::{Executor, MySql, Pool};

pub const INSERT: &str = "INSERT INTO `registrations` (`id`, `created_at`, `updated_at`, `email`, `password_hash`, `code`) VALUES (?, ?, ?, ?, ?, ?)";
pub const SELECT: &str = "SELECT `id`, `created_at`, `updated_at`, `email`, `password_hash`, `code` FROM `registrations` WHERE `id` = ? AND `updated_at` = ?";
pub const UPDATE: &str = "UPDATE `registrations` SET `updated_at` = ?, `code` = ? WHERE `id` = ? AND `updated_at` = ?";
pub const DELETE: &str = "DELETE FROM `registrations` WHERE `id` = ?";

pub async fn init(pool: &Pool<MySql>) {
    hb_log::info(Some("🔧"), "MySQL: Setting up registrations table");

    pool.execute("CREATE TABLE IF NOT EXISTS `registrations` (`id` varchar(36), `created_at` timestamp, `updated_at` timestamp, `email` text, `password_hash` text, `code` text, PRIMARY KEY (`id`))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}
