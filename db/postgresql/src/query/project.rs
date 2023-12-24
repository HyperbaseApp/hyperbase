use sqlx::{Executor, Pool, Postgres};

pub const INSERT: &str = "INSERT INTO \"projects\" (\"id\", \"created_at\", \"updated_at\", \"admin_id\", \"name\") VALUES ($1, $2, $3, $4, $5)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"name\" FROM \"projects\" WHERE \"id\" = $1";
pub const SELECT_MANY_BY_ADMIN_ID :  &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"name\" FROM \"projects\" WHERE \"admin_id\" = $1";
pub const UPDATE: &str = "UPDATE \"projects\" SET \"updated_at\" = $1, \"name\" = $2 WHERE \"id\" = $3";
pub const DELETE: &str = "DELETE FROM \"projects\" WHERE \"id\" = $1";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "PostgreSQL: Setting up projects table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"projects\" (\"id\" uuid, \"created_at\" timestamptz, \"updated_at\" timestamptz, \"admin_id\" uuid, \"name\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(SELECT_MANY_BY_ADMIN_ID).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}
