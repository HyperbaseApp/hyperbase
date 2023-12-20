use sqlx::{Executor, Pool, Sqlite};

pub const INSERT: &str = "INSERT INTO \"admin_password_resets\" (\"id\", \"created_at\", \"updated_at\", \"admin_id\", \"code\") VALUES (?, ?, ?, ?, ?)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"code\" FROM \"admin_password_resets\" WHERE \"id\" = ? AND \"updated_at\" = ?";
pub const UPDATE: &str = "UPDATE \"admin_password_resets\" SET \"updated_at\" = ?, \"code\" = ? WHERE \"id\" = ? AND \"updated_at\" = ?";
pub const DELETE: &str = "DELETE FROM \"admin_password_resets\" WHERE \"id\" = ?";

pub async fn init(pool: &Pool<Sqlite>) {
    hb_log::info(Some("🔧"), "SQLite: Setting up admin_password_resets table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"admin_password_resets\" (\"id\" text, \"created_at\" text, \"updated_at\" text, \"admin_id\" text, \"code\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}
