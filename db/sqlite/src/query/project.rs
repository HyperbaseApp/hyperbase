use sqlx::{Executor, Pool, Sqlite};

pub const INSERT: &str = "INSERT INTO \"projects\" (\"id\", \"created_at\", \"updated_at\", \"admin_id\", \"name\") VALUES (?, ?, ?, ?, ?)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"name\" FROM \"projects\" WHERE \"id\" = ?";
pub const SELECT_MANY_BY_ADMIN_ID :  &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"name\" FROM \"projects\" WHERE \"admin_id\" = ?";
pub const UPDATE: &str = "UPDATE \"projects\" SET \"updated_at\" = ?, \"name\" = ? WHERE \"id\" = ?";
pub const DELETE: &str = "DELETE FROM \"projects\" WHERE \"id\" = ?";

pub async fn init(pool: &Pool<Sqlite>) {
    hb_log::info(Some("🔧"), "SQLite: Setting up projects table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"projects\" (\"id\" text, \"created_at\" text, \"updated_at\" text, \"admin_id\" text, \"name\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(SELECT_MANY_BY_ADMIN_ID).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}
