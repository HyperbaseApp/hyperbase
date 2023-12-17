use sqlx::{Executor, Pool, Postgres};

pub const INSERT: &str = "INSERT INTO \"collections\" (\"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"schema_fields\", \"indexes\") VALUES ($1, $2, $3, $4, $5, $6, $7)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"schema_fields\", \"indexes\" FROM \"collections\" WHERE \"id\" = $1";
pub const SELECT_MANY_BY_PROJECT_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"schema_fields\", \"indexes\" FROM \"collections\" WHERE \"project_id\" = $1";
pub const UPDATE: &str = "UPDATE \"collections\" SET \"updated_at\" = $1, \"name\" = $2, \"schema_fields\" = $3, \"indexes\" = $4 WHERE \"id\" = $5";
pub const DELETE: &str = "DELETE FROM \"collections\" WHERE \"id\" = $1";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "PostgreSQL: Setting up collections table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"collections\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"project_id\" uuid, \"name\" text, \"schema_fields\" jsonb, \"indexes\" text[], PRIMARY KEY (\"id\"))").await.unwrap();

    pool.prepare(INSERT).await.unwrap();
    pool.prepare(SELECT).await.unwrap();
    pool.prepare(SELECT_MANY_BY_PROJECT_ID).await.unwrap();
    pool.prepare(UPDATE).await.unwrap();
    pool.prepare(DELETE).await.unwrap();
}
