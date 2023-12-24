use sqlx::{Executor, Pool, Postgres};

pub const INSERT: &str = "INSERT INTO \"tokens\" (\"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"rules\", \"expired_at\") VALUES ($1, $2, $3, $4, $5, $6, $7)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"rules\", \"expired_at\" FROM \"tokens\" WHERE \"id\" = $1";
pub const SELECT_MANY_BY_ADMIN_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"rules\", \"expired_at\" FROM \"tokens\" WHERE \"admin_id\" = $1";
pub const SELECT_BY_TOKEN: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"rules\", \"expired_at\" FROM \"tokens\" WHERE \"token\" = $1";
pub const UPDATE: &str = "UPDATE \"tokens\" SET \"updated_at\" = $1, \"rules\" = $2, \"expired_at\" = $3 WHERE \"id\" = $4";
pub const DELETE: &str = "DELETE FROM \"tokens\" WHERE \"id\" = $1";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "PostgreSQL: Setting up tokens table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"tokens\" (\"id\" uuid, \"created_at\" timestamptz, \"updated_at\" timestamptz, \"admin_id\" uuid, \"token\" text, \"rules\" jsonb, \"expired_at\" timestamptz, PRIMARY KEY (\"id\"))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(SELECT_MANY_BY_ADMIN_ID).await.unwrap();
    pool.prepare(SELECT_BY_TOKEN).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}
