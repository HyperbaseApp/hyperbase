use anyhow::Result;
use sqlx::{Executor, MySql, Pool};

use crate::{db::MysqlDb, model::local_info::LocalInfoModel};

const INSERT: &str = "INSERT INTO `local_info` (`id`) VALUES (?)";
const SELECT: &str = "SELECT `id` FROM `local_info`";

pub async fn init(pool: &Pool<MySql>) {
    hb_log::info(Some("🔧"), "[MySQL] Setting up local_info table");

    pool.execute("CREATE TABLE IF NOT EXISTS `local_info` (`id` binary(16), PRIMARY KEY (`id`))")
        .await
        .unwrap();
    pool.execute(
        r#"CREATE TRIGGER IF NOT EXISTS only_one_row_insert
    BEFORE INSERT ON `local_info`
    FOR EACH ROW
    BEGIN
      IF (SELECT COUNT(*) FROM `local_info`) >= 1 THEN
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'Only one row allowed in the table';
      END IF;
    END;"#,
    )
    .await
    .unwrap();

    tokio::try_join!(pool.prepare(INSERT), pool.prepare(SELECT),).unwrap();
}

impl MysqlDb {
    pub async fn insert_local_info(&self, value: &LocalInfoModel) -> Result<()> {
        self.execute(sqlx::query(INSERT).bind(value.id())).await?;
        Ok(())
    }

    pub async fn select_local_info(&self) -> Result<LocalInfoModel> {
        Ok(self.fetch_one(sqlx::query_as(SELECT)).await?)
    }
}
