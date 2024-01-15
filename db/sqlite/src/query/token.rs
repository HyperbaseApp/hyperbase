use sqlx::{Executor, Pool, Sqlite};

pub const INSERT: &str = "INSERT INTO \"tokens\" (\"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"bucket_rules\", \"collection_rules\", \"expired_at\") VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"bucket_rules\", \"collection_rules\", \"expired_at\" FROM \"tokens\" WHERE \"id\" = ?";
pub const SELECT_MANY_BY_ADMIN_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"bucket_rules\", \"collection_rules\", \"expired_at\" FROM \"tokens\" WHERE \"admin_id\" = ?";
pub const SELECT_BY_TOKEN: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"bucket_rules\", \"collection_rules\", \"expired_at\" FROM \"tokens\" WHERE \"token\" = ?";
pub const UPDATE: &str = "UPDATE \"tokens\" SET \"updated_at\" = ?, \"bucket_rules\" = ?, \"collection_rules\" = ?, \"expired_at\" = ? WHERE \"id\" = ?";
pub const DELETE: &str = "DELETE FROM \"tokens\" WHERE \"id\" = ?";

pub async fn init(pool: &Pool<Sqlite>) {
    hb_log::info(Some("🔧"), "SQLite: Setting up tokens table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"tokens\" (\"id\" blob, \"created_at\" datetime, \"updated_at\" datetime, \"admin_id\" blob, \"token\" text, \"bucket_rules\" blob, \"bucket_rules\", \"collection_rules\" blob, \"expired_at\" datetime, PRIMARY KEY (\"id\"))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(SELECT_MANY_BY_ADMIN_ID).await.unwrap();
    pool.prepare(SELECT_BY_TOKEN).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}
