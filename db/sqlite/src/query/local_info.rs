use anyhow::Result;
use sqlx::{Executor, Pool, Sqlite};

use crate::{db::SqliteDb, model::local_info::LocalInfoModel};

const INSERT: &str = "INSERT INTO \"local_info\" (\"id\") VALUES (?)";
const SELECT: &str = "SELECT \"id\" FROM \"local_info\"";

pub async fn init(pool: &Pool<Sqlite>) {
    hb_log::info(Some("🔧"), "[SQLite] Setting up local_info table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"local_info\" (\"id\" blob, PRIMARY KEY (\"id\"))")
        .await
        .unwrap();
    pool.execute(
        r#"CREATE TRIGGER only_one_row_insert BEFORE INSERT ON "local_info"
    FOR EACH ROW
    BEGIN
      IF (SELECT COUNT(*) FROM "local_info") >= 1 THEN
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'Table can only have one row';
      END IF;
    END;"#,
    )
    .await
    .unwrap();

    tokio::try_join!(pool.prepare(INSERT), pool.prepare(SELECT),).unwrap();
}

impl SqliteDb {
    pub async fn insert_local_info(&self, value: &LocalInfoModel) -> Result<()> {
        self.execute(sqlx::query(INSERT).bind(value.id())).await?;
        Ok(())
    }

    pub async fn select_local_info(&self) -> Result<LocalInfoModel> {
        Ok(self.fetch_one(sqlx::query_as(SELECT)).await?)
    }
}
