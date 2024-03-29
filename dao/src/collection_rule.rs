use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_mysql::model::collection_rule::CollectionRuleModel as CollectionRuleMysqlModel;
use hb_db_postgresql::model::collection_rule::CollectionRuleModel as CollectionRulePostgresModel;
use hb_db_scylladb::model::collection_rule::CollectionRuleModel as CollectionRuleScyllaModel;
use hb_db_sqlite::model::collection_rule::CollectionRuleModel as CollectionRuleSqliteModel;
use scylla::frame::value::CqlTimestamp as ScyllaCqlTimestamp;
use uuid::Uuid;

use crate::{util::conversion, Db};

pub struct CollectionRuleDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    project_id: Uuid,
    token_id: Uuid,
    collection_id: Uuid,
    find_one: bool,
    find_many: bool,
    insert_one: bool,
    update_one: bool,
    delete_one: bool,
}

impl CollectionRuleDao {
    pub fn new(
        project_id: &Uuid,
        token_id: &Uuid,
        collection_id: &Uuid,
        find_one: &bool,
        find_many: &bool,
        insert_one: &bool,
        update_one: &bool,
        delete_one: &bool,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::now_v7(),
            created_at: now,
            updated_at: now,
            project_id: *project_id,
            token_id: *token_id,
            collection_id: *collection_id,
            find_one: *find_one,
            find_many: *find_many,
            insert_one: *insert_one,
            update_one: *update_one,
            delete_one: *delete_one,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn token_id(&self) -> &Uuid {
        &self.token_id
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn find_one(&self) -> &bool {
        &self.find_one
    }

    pub fn find_many(&self) -> &bool {
        &self.find_many
    }

    pub fn insert_one(&self) -> &bool {
        &self.insert_one
    }

    pub fn update_one(&self) -> &bool {
        &self.update_one
    }

    pub fn delete_one(&self) -> &bool {
        &self.delete_one
    }

    pub fn set_find_one(&mut self, rule: &bool) {
        self.find_one = *rule;
    }

    pub fn set_find_many(&mut self, rule: &bool) {
        self.find_many = *rule;
    }

    pub fn set_insert_one(&mut self, rule: &bool) {
        self.insert_one = *rule;
    }

    pub fn set_update_one(&mut self, rule: &bool) {
        self.update_one = *rule;
    }

    pub fn set_delete_one(&mut self, rule: &bool) {
        self.delete_one = *rule;
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.insert_collection_rule(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.insert_collection_rule(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.insert_collection_rule(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.insert_collection_rule(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_select(db: &Db, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Self::from_scylladb_model(&db.select_collection_rule(id).await?),
            Db::PostgresqlDb(db) => Ok(Self::from_postgresdb_model(
                &db.select_collection_rule(id).await?,
            )),
            Db::MysqlDb(db) => Ok(Self::from_mysqldb_model(
                &db.select_collection_rule(id).await?,
            )),
            Db::SqliteDb(db) => Ok(Self::from_sqlitedb_model(
                &db.select_collection_rule(id).await?,
            )),
        }
    }

    pub async fn db_select_by_token_id_and_collection_id(
        db: &Db,
        token_id: &Uuid,
        collection_id: &Uuid,
    ) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Self::from_scylladb_model(
                &db.select_collection_rule_by_token_id_and_collection_id(token_id, collection_id)
                    .await?,
            ),
            Db::PostgresqlDb(db) => Ok(Self::from_postgresdb_model(
                &db.select_collection_rule_by_token_id_and_collection_id(token_id, collection_id)
                    .await?,
            )),
            Db::MysqlDb(db) => Ok(Self::from_mysqldb_model(
                &db.select_collection_rule_by_token_id_and_collection_id(token_id, collection_id)
                    .await?,
            )),
            Db::SqliteDb(db) => Ok(Self::from_sqlitedb_model(
                &db.select_collection_rule_by_token_id_and_collection_id(token_id, collection_id)
                    .await?,
            )),
        }
    }

    pub async fn db_select_many_by_token_id(db: &Db, token_id: &Uuid) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                let mut collection_rules_data = Vec::new();
                let collection_rules = db
                    .select_many_collection_rules_by_token_id(token_id)
                    .await?;
                for collection_rule in collection_rules {
                    collection_rules_data.push(Self::from_scylladb_model(&collection_rule?)?);
                }
                Ok(collection_rules_data)
            }
            Db::PostgresqlDb(db) => {
                let collection_rules = db
                    .select_many_collection_rules_by_token_id(token_id)
                    .await?;
                let mut collection_rules_data = Vec::with_capacity(collection_rules.len());
                for collection_rule in &collection_rules {
                    collection_rules_data.push(Self::from_postgresdb_model(collection_rule));
                }
                Ok(collection_rules_data)
            }
            Db::MysqlDb(db) => {
                let collection_rules = db
                    .select_many_collection_rules_by_token_id(token_id)
                    .await?;
                let mut collection_rules_data = Vec::with_capacity(collection_rules.len());
                for collection_rule in &collection_rules {
                    collection_rules_data.push(Self::from_mysqldb_model(collection_rule));
                }
                Ok(collection_rules_data)
            }
            Db::SqliteDb(db) => {
                let collection_rules = db
                    .select_many_collection_rules_by_token_id(token_id)
                    .await?;
                let mut collection_rules_data = Vec::with_capacity(collection_rules.len());
                for collection_rule in &collection_rules {
                    collection_rules_data.push(Self::from_sqlitedb_model(collection_rule));
                }
                Ok(collection_rules_data)
            }
        }
    }

    pub async fn db_update(&mut self, db: &Db) -> Result<()> {
        self.updated_at = Utc::now();
        match db {
            Db::ScyllaDb(db) => db.update_collection_rule(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.update_collection_rule(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.update_collection_rule(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.update_collection_rule(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_delete(db: &Db, id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.delete_collection_rule(id).await,
            Db::PostgresqlDb(db) => db.delete_collection_rule(id).await,
            Db::MysqlDb(db) => db.delete_collection_rule(id).await,
            Db::SqliteDb(db) => db.delete_collection_rule(id).await,
        }
    }

    pub async fn db_delete_many_by_token_id(db: &Db, token_id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.delete_many_collection_rules_by_token_id(token_id).await,
            Db::PostgresqlDb(db) => db.delete_many_collection_rules_by_token_id(token_id).await,
            Db::MysqlDb(db) => db.delete_many_collection_rules_by_token_id(token_id).await,
            Db::SqliteDb(db) => db.delete_many_collection_rules_by_token_id(token_id).await,
        }
    }

    fn from_scylladb_model(model: &CollectionRuleScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.created_at())?,
            updated_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.updated_at())?,
            project_id: *model.project_id(),
            token_id: *model.token_id(),
            collection_id: *model.collection_id(),
            find_one: *model.find_one(),
            find_many: *model.find_many(),
            insert_one: *model.insert_one(),
            update_one: *model.update_one(),
            delete_one: *model.delete_one(),
        })
    }

    fn to_scylladb_model(&self) -> CollectionRuleScyllaModel {
        CollectionRuleScyllaModel::new(
            &self.id,
            &ScyllaCqlTimestamp(self.created_at.timestamp_millis()),
            &ScyllaCqlTimestamp(self.updated_at.timestamp_millis()),
            &self.project_id,
            &self.token_id,
            &self.collection_id,
            &self.find_one,
            &self.find_many,
            &self.insert_one,
            &self.update_one,
            &self.delete_one,
        )
    }

    fn from_postgresdb_model(model: &CollectionRulePostgresModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            project_id: *model.project_id(),
            token_id: *model.token_id(),
            collection_id: *model.collection_id(),
            find_one: *model.find_one(),
            find_many: *model.find_many(),
            insert_one: *model.insert_one(),
            update_one: *model.update_one(),
            delete_one: *model.delete_one(),
        }
    }

    fn to_postgresdb_model(&self) -> CollectionRulePostgresModel {
        CollectionRulePostgresModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.project_id,
            &self.token_id,
            &self.collection_id,
            &self.find_one,
            &self.find_many,
            &self.insert_one,
            &self.update_one,
            &self.delete_one,
        )
    }

    fn from_mysqldb_model(model: &CollectionRuleMysqlModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            project_id: *model.project_id(),
            token_id: *model.token_id(),
            collection_id: *model.collection_id(),
            find_one: *model.find_one(),
            find_many: *model.find_many(),
            insert_one: *model.insert_one(),
            update_one: *model.update_one(),
            delete_one: *model.delete_one(),
        }
    }

    fn to_mysqldb_model(&self) -> CollectionRuleMysqlModel {
        CollectionRuleMysqlModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.project_id,
            &self.token_id,
            &self.collection_id,
            &self.find_one,
            &self.find_many,
            &self.insert_one,
            &self.update_one,
            &self.delete_one,
        )
    }

    fn from_sqlitedb_model(model: &CollectionRuleSqliteModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            project_id: *model.project_id(),
            token_id: *model.token_id(),
            collection_id: *model.collection_id(),
            find_one: *model.find_one(),
            find_many: *model.find_many(),
            insert_one: *model.insert_one(),
            update_one: *model.update_one(),
            delete_one: *model.delete_one(),
        }
    }

    fn to_sqlitedb_model(&self) -> CollectionRuleSqliteModel {
        CollectionRuleSqliteModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.project_id,
            &self.token_id,
            &self.collection_id,
            &self.find_one,
            &self.find_many,
            &self.insert_one,
            &self.update_one,
            &self.delete_one,
        )
    }
}
