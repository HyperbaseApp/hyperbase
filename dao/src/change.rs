use std::str::FromStr;

use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use hb_db_mysql::model::change::ChangeModel as ChangeMysqlModel;
use hb_db_postgresql::model::change::ChangeModel as ChangePostgresModel;
use hb_db_sqlite::model::change::ChangeModel as ChangeSqliteModel;
use uuid::Uuid;

use crate::Db;

pub struct ChangeDao {
    table: ChangeTable,
    id: Uuid,
    state: ChangeState,
    updated_at: DateTime<Utc>,
    change_id: Uuid,
}

impl ChangeDao {
    pub fn new(
        table: &ChangeTable,
        id: &Uuid,
        state: &ChangeState,
        updated_at: &DateTime<Utc>,
    ) -> Self {
        Self {
            table: table.to_owned(),
            id: *id,
            state: *state,
            updated_at: *updated_at,
            change_id: Uuid::now_v7(),
        }
    }

    pub fn table(&self) -> &ChangeTable {
        &self.table
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn state(&self) -> &ChangeState {
        &self.state
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    pub fn change_id(&self) -> &Uuid {
        &self.change_id
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(_) => Err(Error::msg("Unimplemented")),
            Db::PostgresqlDb(db) => db.insert_change(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.insert_change(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.insert_change(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_upsert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(_) => Err(Error::msg("Unimplemented")),
            Db::PostgresqlDb(db) => db.upsert_change(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.upsert_change(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.upsert_change(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_select_last_by_table(db: &Db, table: &ChangeTable) -> Result<Option<Self>> {
        match db {
            Db::ScyllaDb(_) => Err(Error::msg("Unimplemented")),
            Db::PostgresqlDb(db) => {
                if let Some(model) = db.select_last_change_by_table(&table.to_string()).await? {
                    Ok(Some(Self::from_postgresdb_model(&model)?))
                } else {
                    Ok(None)
                }
            }
            Db::MysqlDb(db) => {
                if let Some(model) = db.select_last_change_by_table(&table.to_string()).await? {
                    Ok(Some(Self::from_mysqldb_model(&model)?))
                } else {
                    Ok(None)
                }
            }
            Db::SqliteDb(db) => {
                if let Some(model) = db.select_last_change_by_table(&table.to_string()).await? {
                    Ok(Some(Self::from_sqlitedb_model(&model)?))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn from_postgresdb_model(model: &ChangePostgresModel) -> Result<Self> {
        Ok(Self {
            table: ChangeTable::from_str(model.table())?,
            id: *model.id(),
            state: ChangeState::from_str(model.state())?,
            updated_at: *model.updated_at(),
            change_id: *model.change_id(),
        })
    }

    fn to_postgresdb_model(&self) -> ChangePostgresModel {
        ChangePostgresModel::new(
            &self.table.to_string(),
            &self.id,
            self.state.to_str(),
            &self.updated_at,
            &self.change_id,
        )
    }

    fn from_mysqldb_model(model: &ChangeMysqlModel) -> Result<Self> {
        Ok(Self {
            table: ChangeTable::from_str(model.table())?,
            id: *model.id(),
            state: ChangeState::from_str(model.state())?,
            updated_at: *model.updated_at(),
            change_id: *model.change_id(),
        })
    }

    fn to_mysqldb_model(&self) -> ChangeMysqlModel {
        ChangeMysqlModel::new(
            &self.table.to_string(),
            &self.id,
            self.state.to_str(),
            &self.updated_at,
            &self.change_id,
        )
    }

    fn from_sqlitedb_model(model: &ChangeSqliteModel) -> Result<Self> {
        Ok(Self {
            table: ChangeTable::from_str(model.table())?,
            id: *model.id(),
            state: ChangeState::from_str(model.state())?,
            updated_at: *model.updated_at(),
            change_id: *model.change_id(),
        })
    }

    fn to_sqlitedb_model(&self) -> ChangeSqliteModel {
        ChangeSqliteModel::new(
            &self.table.to_string(),
            &self.id,
            self.state.to_str(),
            &self.updated_at,
            &self.change_id,
        )
    }
}

#[derive(Clone, Copy)]
pub enum ChangeTable {
    Admin,
    Project,
    Collection,
    Record(Uuid),
    Bucket,
    File,
    Token,
    CollectionRule,
    BucketRule,
}

impl ChangeTable {
    pub fn to_string(&self) -> String {
        match self {
            Self::Admin => "admins".to_owned(),
            Self::Project => "projects".to_owned(),
            Self::Collection => "collections".to_owned(),
            Self::Record(collection_id) => format!("record_{collection_id}"),
            Self::Bucket => "buckets".to_owned(),
            Self::File => "files".to_owned(),
            Self::Token => "tokens".to_owned(),
            Self::CollectionRule => "collection_rules".to_owned(),
            Self::BucketRule => "bucket_rules".to_owned(),
        }
    }

    pub fn from_str(str: &str) -> Result<Self> {
        match str {
            "admins" => Ok(Self::Admin),
            "projects" => Ok(Self::Project),
            "collections" => Ok(Self::Collection),
            "buckets" => Ok(Self::Bucket),
            "files" => Ok(Self::File),
            "tokens" => Ok(Self::Token),
            "collection_rules" => Ok(Self::CollectionRule),
            "bucket_rules" => Ok(Self::BucketRule),
            str => {
                if str.starts_with("record_") {
                    let collection_id = match Uuid::from_str(&str["record_".len()..]) {
                        Ok(id) => id,
                        Err(err) => return Err(err.into()),
                    };
                    Ok(Self::Record(collection_id))
                } else {
                    Err(Error::msg(format!("Unknown change table '{str}'")))
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum ChangeState {
    Insert,
    Update,
    Delete,
}

impl ChangeState {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Insert => "insert",
            Self::Update => "update",
            Self::Delete => "delete",
        }
    }

    pub fn from_str(str: &str) -> Result<Self> {
        match str {
            "insert" => Ok(Self::Insert),
            "update" => Ok(Self::Update),
            "delete" => Ok(Self::Delete),
            _ => Err(Error::msg(format!("Unknown change state '{str}'"))),
        }
    }
}
