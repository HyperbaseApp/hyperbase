use anyhow::Result;
use sqlx::{Executor, Pool, Postgres};

use crate::{db::PostgresDb, model::local_info::LocalInfoModel};

const INSERT: &str = "INSERT INTO \"local_info\" (\"id\") VALUES ($1)";
const SELECT: &str = "SELECT \"id\" FROM \"local_info\"";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "[PostgreSQL] Setting up local_info table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"local_info\" (\"id\" uuid, PRIMARY KEY (\"id\"))")
        .await
        .unwrap();
    pool.execute(
        r#"CREATE OR REPLACE FUNCTION prevent_duplicate_row() RETURNS TRIGGER AS $$
    BEGIN
      IF (SELECT COUNT(1) FROM "local_info") >= 1 THEN
        RAISE EXCEPTION 'Only one row allowed in the table';
      END IF;
      RETURN NEW;
    END;
    $$ LANGUAGE plpgsql;
    
    CREATE OR REPLACE TRIGGER only_one_row_insert BEFORE INSERT ON "local_info"
    FOR EACH ROW EXECUTE PROCEDURE prevent_duplicate_row();"#,
    )
    .await
    .unwrap();

    tokio::try_join!(pool.prepare(INSERT), pool.prepare(SELECT),).unwrap();
}

impl PostgresDb {
    pub async fn insert_local_info(&self, value: &LocalInfoModel) -> Result<()> {
        self.execute(sqlx::query(INSERT).bind(value.id())).await?;
        Ok(())
    }

    pub async fn select_local_info(&self) -> Result<LocalInfoModel> {
        Ok(self.fetch_one(sqlx::query_as(SELECT)).await?)
    }
}
