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
    timestamp: DateTime<Utc>,
    change_id: Uuid,
}

impl ChangeDao {
    pub fn new(
        table: &ChangeTable,
        id: &Uuid,
        state: &ChangeState,
        timestamp: &DateTime<Utc>,
    ) -> Self {
        Self {
            table: *table,
            id: *id,
            state: *state,
            timestamp: *timestamp,
            change_id: Uuid::now_v7(),
        }
    }

    pub fn raw_new(
        table: &ChangeTable,
        id: &Uuid,
        state: &ChangeState,
        timestamp: &DateTime<Utc>,
        change_id: &Uuid,
    ) -> Self {
        Self {
            table: *table,
            id: *id,
            state: *state,
            timestamp: *timestamp,
            change_id: *change_id,
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

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    pub fn change_id(&self) -> &Uuid {
        &self.change_id
    }

    pub async fn db_insert_or_ignore(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(_) => Err(Error::msg("Unimplemented")),
            Db::PostgresqlDb(db) => {
                db.insert_or_ignore_change(&self.to_postgresdb_model())
                    .await
            }
            Db::MysqlDb(db) => db.insert_or_ignore_change(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.insert_or_ignore_change(&self.to_sqlitedb_model()).await,
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

    pub async fn db_select_by_change_id(db: &Db, change_id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(_) => Err(Error::msg("Unimplemented")),
            Db::PostgresqlDb(db) => {
                Self::from_postgresdb_model(&db.select_change_by_change_id(change_id).await?)
            }
            Db::MysqlDb(db) => {
                Self::from_mysqldb_model(&db.select_change_by_change_id(change_id).await?)
            }
            Db::SqliteDb(db) => {
                Self::from_sqlitedb_model(&db.select_change_by_change_id(change_id).await?)
            }
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

    pub async fn db_select_many_by_change_ids_asc(
        db: &Db,
        change_ids: &Vec<Uuid>,
    ) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(_) => Err(Error::msg("Unimplemented")),
            Db::PostgresqlDb(db) => {
                let changes = db.select_many_changes_by_change_ids_asc(change_ids).await?;
                let mut changes_data = Vec::with_capacity(changes.len());
                for change in &changes {
                    changes_data.push(Self::from_postgresdb_model(change)?);
                }
                Ok(changes_data)
            }
            Db::MysqlDb(db) => {
                let changes = db.select_many_changes_by_change_ids_asc(change_ids).await?;
                let mut changes_data = Vec::with_capacity(changes.len());
                for change in &changes {
                    changes_data.push(Self::from_mysqldb_model(change)?);
                }
                Ok(changes_data)
            }
            Db::SqliteDb(db) => {
                let changes = db.select_many_changes_by_change_ids_asc(change_ids).await?;
                let mut changes_data = Vec::with_capacity(changes.len());
                for change in &changes {
                    changes_data.push(Self::from_sqlitedb_model(change)?);
                }
                Ok(changes_data)
            }
        }
    }

    pub async fn db_select_many_from_timestamp_and_after_change_id_with_limit_asc(
        db: &Db,
        timestamp: &DateTime<Utc>,
        change_id: &Uuid,
        limit: &i32,
    ) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(_) => Err(Error::msg("Unimplemented")),
            Db::PostgresqlDb(db) => {
                let changes = db
                    .select_many_changes_from_timestamp_and_after_change_id_with_limit_asc(
                        timestamp, change_id, limit,
                    )
                    .await?;
                let mut changes_data = Vec::with_capacity(changes.len());
                for change in &changes {
                    changes_data.push(Self::from_postgresdb_model(change)?);
                }
                Ok(changes_data)
            }
            Db::MysqlDb(db) => {
                let changes = db
                    .select_many_changes_from_timestamp_and_after_change_id_with_limit_asc(
                        timestamp, change_id, limit,
                    )
                    .await?;
                let mut changes_data = Vec::with_capacity(changes.len());
                for change in &changes {
                    changes_data.push(Self::from_mysqldb_model(change)?);
                }
                Ok(changes_data)
            }
            Db::SqliteDb(db) => {
                let changes = db
                    .select_many_changes_from_timestamp_and_after_change_id_with_limit_asc(
                        timestamp, change_id, limit,
                    )
                    .await?;
                let mut changes_data = Vec::with_capacity(changes.len());
                for change in &changes {
                    changes_data.push(Self::from_sqlitedb_model(change)?);
                }
                Ok(changes_data)
            }
        }
    }

    fn from_postgresdb_model(model: &ChangePostgresModel) -> Result<Self> {
        Ok(Self {
            table: ChangeTable::from_str(model.table())?,
            id: *model.id(),
            state: ChangeState::from_str(model.state())?,
            timestamp: *model.timestamp(),
            change_id: *model.change_id(),
        })
    }

    fn to_postgresdb_model(&self) -> ChangePostgresModel {
        ChangePostgresModel::new(
            &self.table.to_string(),
            &self.id,
            self.state.to_str(),
            &self.timestamp,
            &self.change_id,
        )
    }

    fn from_mysqldb_model(model: &ChangeMysqlModel) -> Result<Self> {
        Ok(Self {
            table: ChangeTable::from_str(model.table())?,
            id: *model.id(),
            state: ChangeState::from_str(model.state())?,
            timestamp: *model.timestamp(),
            change_id: *model.change_id(),
        })
    }

    fn to_mysqldb_model(&self) -> ChangeMysqlModel {
        ChangeMysqlModel::new(
            &self.table.to_string(),
            &self.id,
            self.state.to_str(),
            &self.timestamp,
            &self.change_id,
        )
    }

    fn from_sqlitedb_model(model: &ChangeSqliteModel) -> Result<Self> {
        Ok(Self {
            table: ChangeTable::from_str(model.table())?,
            id: *model.id(),
            state: ChangeState::from_str(model.state())?,
            timestamp: *model.timestamp(),
            change_id: *model.change_id(),
        })
    }

    fn to_sqlitedb_model(&self) -> ChangeSqliteModel {
        ChangeSqliteModel::new(
            &self.table.to_string(),
            &self.id,
            self.state.to_str(),
            &self.timestamp,
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
    File(Uuid),
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
            Self::Record(collection_id) => format!("records_{collection_id}"),
            Self::Bucket => "buckets".to_owned(),
            Self::File(bucket_id) => format!("files_{bucket_id}"),
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
            "tokens" => Ok(Self::Token),
            "collection_rules" => Ok(Self::CollectionRule),
            "bucket_rules" => Ok(Self::BucketRule),
            str => {
                if str.starts_with("records_") {
                    let collection_id = match Uuid::from_str(&str["records_".len()..]) {
                        Ok(id) => id,
                        Err(err) => return Err(err.into()),
                    };
                    Ok(Self::Record(collection_id))
                } else if str.starts_with("files_") {
                    let bucket_id = match Uuid::from_str(&str["files_".len()..]) {
                        Ok(id) => id,
                        Err(err) => return Err(err.into()),
                    };
                    Ok(Self::File(bucket_id))
                } else {
                    Err(Error::msg(format!("Unknown change table '{str}'")))
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum ChangeState {
    Insert,
    Upsert,
    Update,
    Delete,
}

impl ChangeState {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Insert => "insert",
            Self::Upsert => "upsert",
            Self::Update => "update",
            Self::Delete => "delete",
        }
    }

    pub fn from_str(str: &str) -> Result<Self> {
        match str {
            "insert" => Ok(Self::Insert),
            "upsert" => Ok(Self::Upsert),
            "update" => Ok(Self::Update),
            "delete" => Ok(Self::Delete),
            _ => Err(Error::msg(format!("Unknown change state '{str}'"))),
        }
    }
}
