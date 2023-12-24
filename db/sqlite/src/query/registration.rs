use sqlx::{Executor, Pool, Sqlite};

pub const INSERT: &str = "INSERT INTO \"registrations\" (\"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\", \"code\") VALUES (?, ?, ?, ?, ?, ?)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\", \"code\" FROM \"registrations\" WHERE \"id\" = ? AND \"updated_at\" >= ?";
pub const UPDATE: &str = "UPDATE \"registrations\" SET \"updated_at\" = ?, \"code\" = ? WHERE \"id\" = ? AND \"updated_at\" >= ?";
pub const DELETE: &str = "DELETE FROM \"registrations\" WHERE \"id\" = ?";

pub async fn init(pool: &Pool<Sqlite>) {
    hb_log::info(Some("🔧"), "SQLite: Setting up registrations table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"registrations\" (\"id\" blob, \"created_at\" datetime, \"updated_at\" datetime, \"email\" text, \"password_hash\" text, \"code\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}
