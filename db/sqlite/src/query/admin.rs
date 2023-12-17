use sqlx::{Executor, Pool, Sqlite};

pub const INSERT: &str = "INSERT INTO \"admins\" (\"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\") VALUES (?, ?, ?, ?, ?)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\" FROM \"admins\" WHERE \"id\" = ?";
pub const SELECT_BY_EMAIL: &str= "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\" FROM \"admins\" WHERE \"email\" = ?";
pub const UPDATE: &str = "UPDATE \"admins\" SET \"updated_at\" = ?, \"email\" = ?, \"password_hash\" = ? WHERE \"id\" = ?";
pub const DELETE: &str = "DELETE FROM \"admins\" WHERE \"id\" = ?";

pub async fn init(pool: &Pool<Sqlite>) {
    hb_log::info(Some("🔧"), "SQLite: Setting up admins table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"admins\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"email\" text, \"password_hash\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(SELECT_BY_EMAIL).await.unwrap();
    pool.prepare(SELECT_BY_EMAIL).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}
