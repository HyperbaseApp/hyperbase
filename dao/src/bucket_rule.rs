use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use hb_db_mysql::model::bucket_rule::BucketRuleModel as BucketRuleMysqlModel;
use hb_db_postgresql::model::bucket_rule::BucketRuleModel as BucketRulePostgresModel;
use hb_db_scylladb::model::bucket_rule::BucketRuleModel as BucketRuleScyllaModel;
use hb_db_sqlite::model::bucket_rule::BucketRuleModel as BucketRuleSqliteModel;
use uuid::Uuid;

use crate::{util::conversion, Db};

pub struct BucketRuleDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    project_id: Uuid,
    token_id: Uuid,
    bucket_id: Uuid,
    find_one: BucketPermission,
    find_many: BucketPermission,
    insert_one: bool,
    update_one: BucketPermission,
    delete_one: BucketPermission,
}

impl BucketRuleDao {
    pub fn new(
        project_id: &Uuid,
        token_id: &Uuid,
        bucket_id: &Uuid,
        find_one: &BucketPermission,
        find_many: &BucketPermission,
        insert_one: &bool,
        update_one: &BucketPermission,
        delete_one: &BucketPermission,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::now_v7(),
            created_at: now,
            updated_at: now,
            project_id: *project_id,
            token_id: *token_id,
            bucket_id: *bucket_id,
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

    pub fn bucket_id(&self) -> &Uuid {
        &self.bucket_id
    }

    pub fn find_one(&self) -> &BucketPermission {
        &self.find_one
    }

    pub fn find_many(&self) -> &BucketPermission {
        &self.find_many
    }

    pub fn insert_one(&self) -> &bool {
        &self.insert_one
    }

    pub fn update_one(&self) -> &BucketPermission {
        &self.update_one
    }

    pub fn delete_one(&self) -> &BucketPermission {
        &self.delete_one
    }

    pub fn set_find_one(&mut self, rule: &BucketPermission) {
        self.find_one = *rule;
    }

    pub fn set_find_many(&mut self, rule: &BucketPermission) {
        self.find_many = *rule;
    }

    pub fn set_insert_one(&mut self, rule: &bool) {
        self.insert_one = *rule;
    }

    pub fn set_update_one(&mut self, rule: &BucketPermission) {
        self.update_one = *rule;
    }

    pub fn set_delete_one(&mut self, rule: &BucketPermission) {
        self.delete_one = *rule;
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.insert_bucket_rule(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.insert_bucket_rule(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.insert_bucket_rule(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.insert_bucket_rule(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_select(db: &Db, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Self::from_scylladb_model(&db.select_bucket_rule(id).await?),
            Db::PostgresqlDb(db) => Self::from_postgresdb_model(&db.select_bucket_rule(id).await?),
            Db::MysqlDb(db) => Self::from_mysqldb_model(&db.select_bucket_rule(id).await?),
            Db::SqliteDb(db) => Self::from_sqlitedb_model(&db.select_bucket_rule(id).await?),
        }
    }

    pub async fn db_select_by_token_id_and_bucket_id(
        db: &Db,
        token_id: &Uuid,
        bucket_id: &Uuid,
    ) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Self::from_scylladb_model(
                &db.select_bucket_rule_by_token_id_and_bucket_id(token_id, bucket_id)
                    .await?,
            ),
            Db::PostgresqlDb(db) => Self::from_postgresdb_model(
                &db.select_bucket_rule_by_token_id_and_bucket_id(token_id, bucket_id)
                    .await?,
            ),
            Db::MysqlDb(db) => Self::from_mysqldb_model(
                &db.select_bucket_rule_by_token_id_and_bucket_id(token_id, bucket_id)
                    .await?,
            ),
            Db::SqliteDb(db) => Self::from_sqlitedb_model(
                &db.select_bucket_rule_by_token_id_and_bucket_id(token_id, bucket_id)
                    .await?,
            ),
        }
    }

    pub async fn db_select_many_by_token_id(db: &Db, token_id: &Uuid) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                let mut bucket_rules_data = Vec::new();
                let bucket_rules = db.select_many_bucket_rules_by_token_id(token_id).await?;
                for bucket_rule in bucket_rules {
                    bucket_rules_data.push(Self::from_scylladb_model(&bucket_rule?)?);
                }
                Ok(bucket_rules_data)
            }
            Db::PostgresqlDb(db) => {
                let bucket_rules = db.select_many_bucket_rules_by_token_id(token_id).await?;
                let mut bucket_rules_data = Vec::with_capacity(bucket_rules.len());
                for bucket_rule in &bucket_rules {
                    bucket_rules_data.push(Self::from_postgresdb_model(bucket_rule)?);
                }
                Ok(bucket_rules_data)
            }
            Db::MysqlDb(db) => {
                let bucket_rules = db.select_many_bucket_rules_by_token_id(token_id).await?;
                let mut bucket_rules_data = Vec::with_capacity(bucket_rules.len());
                for bucket_rule in &bucket_rules {
                    bucket_rules_data.push(Self::from_mysqldb_model(bucket_rule)?);
                }
                Ok(bucket_rules_data)
            }
            Db::SqliteDb(db) => {
                let bucket_rules = db.select_many_bucket_rules_by_token_id(token_id).await?;
                let mut bucket_rules_data = Vec::with_capacity(bucket_rules.len());
                for bucket_rule in &bucket_rules {
                    bucket_rules_data.push(Self::from_sqlitedb_model(bucket_rule)?);
                }
                Ok(bucket_rules_data)
            }
        }
    }

    pub async fn db_select_many_after_id_with_limit(
        db: &Db,
        after_id: &Option<Uuid>,
        limit: &i32,
    ) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                let mut bucket_rules_data = Vec::new();
                let bucket_rules = db
                    .select_many_bucket_rules_after_id_with_limit(after_id, limit)
                    .await?;
                for bucket_rule in bucket_rules {
                    bucket_rules_data.push(Self::from_scylladb_model(&bucket_rule?)?);
                }
                Ok(bucket_rules_data)
            }
            Db::PostgresqlDb(db) => {
                let bucket_rules = db
                    .select_many_bucket_rules_after_id_with_limit(after_id, limit)
                    .await?;
                let mut bucket_rules_data = Vec::with_capacity(bucket_rules.len());
                for bucket_rule in &bucket_rules {
                    bucket_rules_data.push(Self::from_postgresdb_model(bucket_rule)?);
                }
                Ok(bucket_rules_data)
            }
            Db::MysqlDb(db) => {
                let bucket_rules = db
                    .select_many_bucket_rules_after_id_with_limit(after_id, limit)
                    .await?;
                let mut bucket_rules_data = Vec::with_capacity(bucket_rules.len());
                for bucket_rule in &bucket_rules {
                    bucket_rules_data.push(Self::from_mysqldb_model(bucket_rule)?);
                }
                Ok(bucket_rules_data)
            }
            Db::SqliteDb(db) => {
                let bucket_rules = db
                    .select_many_bucket_rules_after_id_with_limit(after_id, limit)
                    .await?;
                let mut bucket_rules_data = Vec::with_capacity(bucket_rules.len());
                for bucket_rule in &bucket_rules {
                    bucket_rules_data.push(Self::from_sqlitedb_model(bucket_rule)?);
                }
                Ok(bucket_rules_data)
            }
        }
    }

    pub async fn db_update(&mut self, db: &Db) -> Result<()> {
        self.updated_at = Utc::now();
        match db {
            Db::ScyllaDb(db) => db.update_bucket_rule(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.update_bucket_rule(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.update_bucket_rule(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.update_bucket_rule(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_delete(db: &Db, id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.delete_bucket_rule(id).await,
            Db::PostgresqlDb(db) => db.delete_bucket_rule(id).await,
            Db::MysqlDb(db) => db.delete_bucket_rule(id).await,
            Db::SqliteDb(db) => db.delete_bucket_rule(id).await,
        }
    }

    pub async fn db_delete_many_by_token_id(db: &Db, token_id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.delete_many_bucket_rules_by_token_id(token_id).await,
            Db::PostgresqlDb(db) => db.delete_many_bucket_rules_by_token_id(token_id).await,
            Db::MysqlDb(db) => db.delete_many_bucket_rules_by_token_id(token_id).await,
            Db::SqliteDb(db) => db.delete_many_bucket_rules_by_token_id(token_id).await,
        }
    }

    pub async fn db_delete_many_by_bucket_id(db: &Db, bucket_id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.delete_many_bucket_rules_by_bucket_id(bucket_id).await,
            Db::PostgresqlDb(db) => db.delete_many_bucket_rules_by_bucket_id(bucket_id).await,
            Db::MysqlDb(db) => db.delete_many_bucket_rules_by_bucket_id(bucket_id).await,
            Db::SqliteDb(db) => db.delete_many_bucket_rules_by_bucket_id(bucket_id).await,
        }
    }

    fn from_scylladb_model(model: &BucketRuleScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.created_at())?,
            updated_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.updated_at())?,
            project_id: *model.project_id(),
            token_id: *model.token_id(),
            bucket_id: *model.bucket_id(),
            find_one: BucketPermission::from_str(model.find_one())?,
            find_many: BucketPermission::from_str(model.find_many())?,
            insert_one: *model.insert_one(),
            update_one: BucketPermission::from_str(model.update_one())?,
            delete_one: BucketPermission::from_str(model.delete_one())?,
        })
    }

    fn to_scylladb_model(&self) -> BucketRuleScyllaModel {
        BucketRuleScyllaModel::new(
            &self.id,
            &conversion::datetime_utc_to_scylla_cql_timestamp(&self.created_at),
            &conversion::datetime_utc_to_scylla_cql_timestamp(&self.updated_at),
            &self.project_id,
            &self.token_id,
            &self.bucket_id,
            self.find_one.to_str(),
            self.find_many.to_str(),
            &self.insert_one,
            self.update_one.to_str(),
            self.delete_one.to_str(),
        )
    }

    fn from_postgresdb_model(model: &BucketRulePostgresModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            project_id: *model.project_id(),
            token_id: *model.token_id(),
            bucket_id: *model.bucket_id(),
            find_one: BucketPermission::from_str(model.find_one())?,
            find_many: BucketPermission::from_str(model.find_many())?,
            insert_one: *model.insert_one(),
            update_one: BucketPermission::from_str(model.update_one())?,
            delete_one: BucketPermission::from_str(model.delete_one())?,
        })
    }

    fn to_postgresdb_model(&self) -> BucketRulePostgresModel {
        BucketRulePostgresModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.project_id,
            &self.token_id,
            &self.bucket_id,
            self.find_one.to_str(),
            self.find_many.to_str(),
            &self.insert_one,
            self.update_one.to_str(),
            self.delete_one.to_str(),
        )
    }

    fn from_mysqldb_model(model: &BucketRuleMysqlModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            project_id: *model.project_id(),
            token_id: *model.token_id(),
            bucket_id: *model.bucket_id(),
            find_one: BucketPermission::from_str(model.find_one())?,
            find_many: BucketPermission::from_str(model.find_many())?,
            insert_one: *model.insert_one(),
            update_one: BucketPermission::from_str(model.update_one())?,
            delete_one: BucketPermission::from_str(model.delete_one())?,
        })
    }

    fn to_mysqldb_model(&self) -> BucketRuleMysqlModel {
        BucketRuleMysqlModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.project_id,
            &self.token_id,
            &self.bucket_id,
            self.find_one.to_str(),
            self.find_many.to_str(),
            &self.insert_one,
            self.update_one.to_str(),
            self.delete_one.to_str(),
        )
    }

    fn from_sqlitedb_model(model: &BucketRuleSqliteModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            project_id: *model.project_id(),
            token_id: *model.token_id(),
            bucket_id: *model.bucket_id(),
            find_one: BucketPermission::from_str(model.find_one())?,
            find_many: BucketPermission::from_str(model.find_many())?,
            insert_one: *model.insert_one(),
            update_one: BucketPermission::from_str(model.update_one())?,
            delete_one: BucketPermission::from_str(model.delete_one())?,
        })
    }

    fn to_sqlitedb_model(&self) -> BucketRuleSqliteModel {
        BucketRuleSqliteModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.project_id,
            &self.token_id,
            &self.bucket_id,
            self.find_one.to_str(),
            self.find_many.to_str(),
            &self.insert_one,
            self.update_one.to_str(),
            self.delete_one.to_str(),
        )
    }
}

#[derive(Clone, Copy)]
pub enum BucketPermission {
    All,
    SelfMade,
    None,
}

impl BucketPermission {
    pub fn to_str(&self) -> &str {
        match self {
            Self::All => "all",
            Self::SelfMade => "self_made",
            Self::None => "none",
        }
    }

    pub fn from_str(str: &str) -> Result<Self> {
        match str {
            "all" => Ok(Self::All),
            "self_made" => Ok(Self::SelfMade),
            "none" => Ok(Self::None),
            _ => Err(Error::msg(format!("Unknown bucket permission '{str}'"))),
        }
    }
}
