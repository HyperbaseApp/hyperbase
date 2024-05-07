use anyhow::{Error, Result};
use hb_db_mysql::model::local_info::LocalInfoModel as LocalInfoMysqlModel;
use hb_db_postgresql::model::local_info::LocalInfoModel as LocalInfoPostgresModel;
use hb_db_sqlite::model::local_info::LocalInfoModel as LocalInfoSqliteModel;
use uuid::Uuid;

use crate::Db;

pub struct LocalInfoDao {
    id: Uuid,
}

impl LocalInfoDao {
    pub fn new() -> Self {
        Self { id: Uuid::now_v7() }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(_) => Err(Error::msg("Unimplemented")),
            Db::PostgresqlDb(db) => db.insert_local_info(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.insert_local_info(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.insert_local_info(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_select(db: &Db) -> Result<Self> {
        match db {
            Db::ScyllaDb(_) => Err(Error::msg("Unimplemented")),
            Db::PostgresqlDb(db) => Ok(Self::from_postgresdb_model(&db.select_local_info().await?)),
            Db::MysqlDb(db) => Ok(Self::from_mysqldb_model(&db.select_local_info().await?)),
            Db::SqliteDb(db) => Ok(Self::from_sqlitedb_model(&db.select_local_info().await?)),
        }
    }

    fn from_postgresdb_model(model: &LocalInfoPostgresModel) -> Self {
        Self { id: *model.id() }
    }

    fn to_postgresdb_model(&self) -> LocalInfoPostgresModel {
        LocalInfoPostgresModel::new(&self.id)
    }

    fn from_mysqldb_model(model: &LocalInfoMysqlModel) -> Self {
        Self { id: *model.id() }
    }

    fn to_mysqldb_model(&self) -> LocalInfoMysqlModel {
        LocalInfoMysqlModel::new(&self.id)
    }

    fn from_sqlitedb_model(model: &LocalInfoSqliteModel) -> Self {
        Self { id: *model.id() }
    }

    fn to_sqlitedb_model(&self) -> LocalInfoSqliteModel {
        LocalInfoSqliteModel::new(&self.id)
    }
}
