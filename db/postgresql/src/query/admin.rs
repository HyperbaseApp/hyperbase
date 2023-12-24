use sqlx::{Executor, Pool, Postgres};

pub const INSERT: &str = "INSERT INTO \"admins\" (\"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\") VALUES ($1, $2, $3, $4, $5)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\" FROM \"admins\" WHERE \"id\" = $1";
pub const SELECT_BY_EMAIL: &str= "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\" FROM \"admins\" WHERE \"email\" = $1";
pub const UPDATE: &str = "UPDATE \"admins\" SET \"updated_at\" = $1, \"email\" = $2, \"password_hash\" = $3 WHERE \"id\" = $4";
pub const DELETE: &str = "DELETE FROM \"admins\" WHERE \"id\" = $1";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "PostgreSQL: Setting up admins table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"admins\" (\"id\" uuid, \"created_at\" timestamptz, \"updated_at\" timestamptz, \"email\" text, \"password_hash\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(SELECT_BY_EMAIL).await.unwrap();
    pool.prepare(SELECT_BY_EMAIL).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}
