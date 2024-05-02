use admin::AdminDao;
use anyhow::Result;
use change::{ChangeDao, ChangeState, ChangeTable};
use hb_db_mysql::db::MysqlDb;
use hb_db_postgresql::db::PostgresDb;
use hb_db_scylladb::db::ScyllaDb;
use hb_db_sqlite::db::SqliteDb;

pub mod admin;
pub mod admin_password_reset;
pub mod bucket;
pub mod bucket_rule;
pub mod change;
pub mod collection;
pub mod collection_rule;
pub mod file;
pub mod log;
pub mod project;
pub mod record;
pub mod registration;
pub mod remote_sync;
pub mod token;
mod util;
pub mod value;

pub enum Db {
    ScyllaDb(ScyllaDb),
    PostgresqlDb(PostgresDb),
    MysqlDb(MysqlDb),
    SqliteDb(SqliteDb),
}

impl Db {
    pub async fn init(&self) -> Result<()> {
        hb_log::info(Some("⚡"), "[DAO] Initializing component");

        self.update_change().await?;
        Ok(())
    }

    async fn update_change(&self) -> Result<()> {
        hb_log::info(None, "[DAO] Updating changes table entries");

        match ChangeDao::db_select_last(self).await? {
            Some(change_data) => todo!(),
            None => {
                let mut last_admin_id = None;
                loop {
                    let admins_data =
                        AdminDao::db_select_many_after_id_with_limit(self, &last_admin_id, &30)
                            .await?;
                    match admins_data.last() {
                        Some(admin_data) => {
                            last_admin_id = Some(*admin_data.id());
                        }
                        None => break,
                    }
                    for admin_data in &admins_data {
                        ChangeDao::new(&ChangeTable::Admin, admin_data.id(), &ChangeState::Insert)
                            .db_insert(self)
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }
}
