use sqlx::{Executor, Pool, Postgres};

pub const INSERT: &str = "INSERT INTO \"registrations\" (\"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\", \"code\") VALUES ($1, $2, $3, $4, $5, $6)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\", \"code\" FROM \"registrations\" WHERE \"id\" = $1 AND \"updated_at\" >= $2";
pub const UPDATE: &str = "UPDATE \"registrations\" SET \"updated_at\" = $1, \"code\" = $2 WHERE \"id\" = $3 AND \"updated_at\" >= $4";
pub const DELETE: &str = "DELETE FROM \"registrations\" WHERE \"id\" = $1";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "PostgreSQL: Setting up registrations table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"registrations\" (\"id\" uuid, \"created_at\" timestamptz, \"updated_at\" timestamptz, \"email\" text, \"password_hash\" text, \"code\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}
