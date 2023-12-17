use sqlx::{Executor, Pool, Postgres};

pub const INSERT: &str = "INSERT INTO \"admin_password_resets\" (\"id\", \"created_at\", \"updated_at\", \"admin_id\", \"code\") VALUES ($1, $2, $3, $4, $5)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"code\" FROM \"admin_password_resets\" WHERE \"id\" = $1 AND \"updated_at\" = $2";
pub const UPDATE: &str = "UPDATE \"admin_password_resets\" SET \"updated_at\" = $1, \"code\" = $2 WHERE \"id\" = $3 AND \"updated_at\" = $4";
pub const DELETE: &str = "DELETE FROM \"admin_password_resets\" WHERE \"id\" = $1";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(
        Some("🔧"),
        "PostgreSQL: Setting up admin_password_resets table",
    );

    pool.execute("CREATE TABLE IF NOT EXISTS \"admin_password_resets\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"admin_id\" uuid, \"code\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}
