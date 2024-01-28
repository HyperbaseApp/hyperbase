use ahash::{HashMap, HashMapExt};
use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_mysql::model::token::{
    TokenBucketRuleMethodModel as TokenBucketRuleMethodMysqlModel,
    TokenCollectionRuleMethodModel as TokenCollectionRuleMethodMysqlModel,
    TokenModel as TokenMysqlModel,
};
use hb_db_postgresql::model::token::{
    TokenBucketRuleMethodModel as TokenBucketRuleMethodPostgresModel,
    TokenCollectionRuleMethodModel as TokenCollectionRuleMethodPostgresModel,
    TokenModel as TokenPostgresModel,
};
use hb_db_scylladb::model::token::{
    TokenBucketRuleMethodModel as TokenBucketRuleMethodScyllaModel,
    TokenCollectionRuleMethodModel as TokenCollectionRuleMethodScyllaModel,
    TokenModel as TokenScyllaModel,
};
use hb_db_sqlite::model::token::{
    TokenBucketRuleMethodModel as TokenBucketRuleMethodSqliteModel,
    TokenCollectionRuleMethodModel as TokenCollectionRuleMethodSqliteModel,
    TokenModel as TokenSqliteModel,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use scylla::frame::value::CqlTimestamp as ScyllaCqlTimestamp;
use serde::Serialize;
use uuid::Uuid;

use crate::{util::conversion, Db};

pub struct TokenDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    admin_id: Uuid,
    token: String,
    bucket_rules: HashMap<Uuid, TokenBucketRuleMethod>,
    collection_rules: HashMap<Uuid, TokenCollectionRuleMethod>,
    expired_at: Option<DateTime<Utc>>,
}

impl TokenDao {
    pub fn new(
        admin_id: &Uuid,
        token_length: &usize,
        bucket_rules: &HashMap<Uuid, TokenBucketRuleMethod>,
        collection_rules: &HashMap<Uuid, TokenCollectionRuleMethod>,
        expired_at: &Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            created_at: now,
            updated_at: now,
            admin_id: *admin_id,
            token: thread_rng()
                .sample_iter(&Alphanumeric)
                .take(*token_length)
                .map(char::from)
                .collect(),
            bucket_rules: bucket_rules.clone(),
            collection_rules: collection_rules.clone(),
            expired_at: *expired_at,
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

    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn bucket_rules(&self) -> &HashMap<Uuid, TokenBucketRuleMethod> {
        &self.bucket_rules
    }

    pub fn collection_rules(&self) -> &HashMap<Uuid, TokenCollectionRuleMethod> {
        &self.collection_rules
    }

    pub fn expired_at(&self) -> &Option<DateTime<Utc>> {
        &self.expired_at
    }

    pub fn set_collection_rules(
        &mut self,
        collection_rules: &HashMap<Uuid, TokenCollectionRuleMethod>,
    ) {
        self.collection_rules = collection_rules.clone();
    }

    pub fn set_expired_at(&mut self, expired_at: &Option<DateTime<Utc>>) {
        self.expired_at = *expired_at;
    }

    pub fn is_allow_find_one_file(&self, bucket_id: &Uuid) -> bool {
        match self.bucket_rules.get(bucket_id) {
            Some(bucket_rules) => bucket_rules.find_one,
            None => false,
        }
    }

    pub fn is_allow_find_many_files(&self, bucket_id: &Uuid) -> bool {
        match self.bucket_rules.get(bucket_id) {
            Some(bucket_rules) => bucket_rules.find_many,
            None => false,
        }
    }

    pub fn is_allow_insert_file(&self, bucket_id: &Uuid) -> bool {
        match self.bucket_rules.get(bucket_id) {
            Some(bucket_rules) => bucket_rules.insert,
            None => false,
        }
    }

    pub fn is_allow_update_file(&self, bucket_id: &Uuid) -> bool {
        match self.bucket_rules.get(bucket_id) {
            Some(bucket_rules) => bucket_rules.update,
            None => false,
        }
    }

    pub fn is_allow_delete_file(&self, bucket_id: &Uuid) -> bool {
        match self.bucket_rules.get(bucket_id) {
            Some(bucket_rules) => bucket_rules.delete,
            None => false,
        }
    }

    pub fn is_allow_find_one_record(&self, collection_id: &Uuid) -> bool {
        match self.collection_rules.get(collection_id) {
            Some(collection_rules) => collection_rules.find_one,
            None => false,
        }
    }

    pub fn is_allow_find_many_records(&self, collection_id: &Uuid) -> bool {
        match self.collection_rules.get(collection_id) {
            Some(collection_rules) => collection_rules.find_many,
            None => false,
        }
    }

    pub fn is_allow_insert_record(&self, collection_id: &Uuid) -> bool {
        match self.collection_rules.get(collection_id) {
            Some(collection_rules) => collection_rules.insert,
            None => false,
        }
    }

    pub fn is_allow_update_record(&self, collection_id: &Uuid) -> bool {
        match self.collection_rules.get(collection_id) {
            Some(collection_rules) => collection_rules.update,
            None => false,
        }
    }

    pub fn is_allow_delete_record(&self, collection_id: &Uuid) -> bool {
        match self.collection_rules.get(collection_id) {
            Some(collection_rules) => collection_rules.delete,
            None => false,
        }
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.insert_token(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.insert_token(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.insert_token(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.insert_token(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_select(db: &Db, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Self::from_scylladb_model(&db.select_token(id).await?),
            Db::PostgresqlDb(db) => Ok(Self::from_postgresdb_model(&db.select_token(id).await?)),
            Db::MysqlDb(db) => Ok(Self::from_mysqldb_model(&db.select_token(id).await?)),
            Db::SqliteDb(db) => Ok(Self::from_sqlitedb_model(&db.select_token(id).await?)),
        }
    }

    pub async fn db_select_many_by_admin_id(db: &Db, admin_id: &Uuid) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                let mut tokens_data = Vec::new();
                for token in db.select_many_tokens_by_admin_id(admin_id).await? {
                    tokens_data.push(Self::from_scylladb_model(&token?)?);
                }
                Ok(tokens_data)
            }
            Db::PostgresqlDb(db) => Ok(db
                .select_many_tokens_by_admin_id(admin_id)
                .await?
                .iter()
                .map(|data| Self::from_postgresdb_model(data))
                .collect()),
            Db::MysqlDb(db) => Ok(db
                .select_many_tokens_by_admin_id(admin_id)
                .await?
                .iter()
                .map(|data| Self::from_mysqldb_model(data))
                .collect()),
            Db::SqliteDb(db) => Ok(db
                .select_many_tokens_by_admin_id(admin_id)
                .await?
                .iter()
                .map(|data| Self::from_sqlitedb_model(data))
                .collect()),
        }
    }

    pub async fn db_update(&mut self, db: &Db) -> Result<()> {
        self.updated_at = Utc::now();
        match db {
            Db::ScyllaDb(db) => db.update_token(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.update_token(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.update_token(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.update_token(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_delete(db: &Db, id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.delete_token(id).await,
            Db::PostgresqlDb(db) => db.delete_token(id).await,
            Db::MysqlDb(db) => db.delete_token(id).await,
            Db::SqliteDb(db) => db.delete_token(id).await,
        }
    }

    fn from_scylladb_model(model: &TokenScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.created_at())?,
            updated_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.updated_at())?,
            admin_id: *model.admin_id(),
            token: model.token().to_owned(),
            bucket_rules: match model.bucket_rules() {
                Some(bucket_rules) => bucket_rules
                    .iter()
                    .map(|(bucket_id, bucket_rules)| {
                        (
                            *bucket_id,
                            TokenBucketRuleMethod::from_scylladb_model(bucket_rules),
                        )
                    })
                    .collect(),
                None => HashMap::new(),
            },
            collection_rules: match model.collection_rules() {
                Some(collection_rules) => collection_rules
                    .iter()
                    .map(|(collection_id, collection_rules)| {
                        (
                            *collection_id,
                            TokenCollectionRuleMethod::from_scylladb_model(collection_rules),
                        )
                    })
                    .collect(),
                None => HashMap::new(),
            },
            expired_at: match &model.expired_at() {
                Some(expired_at) => Some(conversion::scylla_cql_timestamp_to_datetime_utc(
                    expired_at,
                )?),
                None => None,
            },
        })
    }

    fn to_scylladb_model(&self) -> TokenScyllaModel {
        TokenScyllaModel::new(
            &self.id,
            &ScyllaCqlTimestamp(self.created_at.timestamp_millis()),
            &ScyllaCqlTimestamp(self.updated_at.timestamp_millis()),
            &self.admin_id,
            &self.token,
            &Some(
                self.bucket_rules
                    .iter()
                    .map(|(bucket_id, bucket_rules)| (*bucket_id, bucket_rules.to_scylladb_model()))
                    .collect(),
            ),
            &Some(
                self.collection_rules
                    .iter()
                    .map(|(collection_id, collection_rules)| {
                        (*collection_id, collection_rules.to_scylladb_model())
                    })
                    .collect(),
            ),
            &match &self.expired_at {
                Some(expired_at) => Some(ScyllaCqlTimestamp(expired_at.timestamp_millis())),
                None => None,
            },
        )
    }

    fn from_postgresdb_model(model: &TokenPostgresModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            token: model.token().to_owned(),
            bucket_rules: model
                .bucket_rules()
                .iter()
                .map(|(bucket_id, bucket_rules)| {
                    (
                        *bucket_id,
                        TokenBucketRuleMethod::from_postgresdb_model(bucket_rules),
                    )
                })
                .collect(),
            collection_rules: model
                .collection_rules()
                .iter()
                .map(|(collection_id, collection_rules)| {
                    (
                        *collection_id,
                        TokenCollectionRuleMethod::from_postgresdb_model(collection_rules),
                    )
                })
                .collect(),
            expired_at: *model.expired_at(),
        }
    }

    fn to_postgresdb_model(&self) -> TokenPostgresModel {
        TokenPostgresModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.admin_id,
            &self.token,
            &sqlx::types::Json(
                self.bucket_rules
                    .iter()
                    .map(|(bucket_id, bucket_rules)| {
                        (*bucket_id, bucket_rules.to_postgresdb_model())
                    })
                    .collect(),
            ),
            &sqlx::types::Json(
                self.collection_rules
                    .iter()
                    .map(|(collection_id, collection_rules)| {
                        (*collection_id, collection_rules.to_postgresdb_model())
                    })
                    .collect(),
            ),
            &self.expired_at,
        )
    }

    fn from_mysqldb_model(model: &TokenMysqlModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            token: model.token().to_owned(),
            bucket_rules: model
                .bucket_rules()
                .iter()
                .map(|(bucket_id, bucket_rules)| {
                    (
                        *bucket_id,
                        TokenBucketRuleMethod::from_mysqldb_model(bucket_rules),
                    )
                })
                .collect(),
            collection_rules: model
                .collection_rules()
                .iter()
                .map(|(collection_id, collection_rules)| {
                    (
                        *collection_id,
                        TokenCollectionRuleMethod::from_mysqldb_model(collection_rules),
                    )
                })
                .collect(),
            expired_at: *model.expired_at(),
        }
    }

    fn to_mysqldb_model(&self) -> TokenMysqlModel {
        TokenMysqlModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.admin_id,
            &self.token,
            &sqlx::types::Json(
                self.bucket_rules
                    .iter()
                    .map(|(bucket_id, bucket_rules)| (*bucket_id, bucket_rules.to_mysqldb_model()))
                    .collect(),
            ),
            &sqlx::types::Json(
                self.collection_rules
                    .iter()
                    .map(|(collection_id, collection_rules)| {
                        (*collection_id, collection_rules.to_mysqldb_model())
                    })
                    .collect(),
            ),
            &self.expired_at,
        )
    }

    fn from_sqlitedb_model(model: &TokenSqliteModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            token: model.token().to_owned(),
            bucket_rules: model
                .bucket_rules()
                .iter()
                .map(|(bucket_id, bucket_rules)| {
                    (
                        *bucket_id,
                        TokenBucketRuleMethod::from_sqlitedb_model(bucket_rules),
                    )
                })
                .collect(),
            collection_rules: model
                .collection_rules()
                .iter()
                .map(|(collection_id, collection_rules)| {
                    (
                        *collection_id,
                        TokenCollectionRuleMethod::from_sqlitedb_model(collection_rules),
                    )
                })
                .collect(),
            expired_at: *model.expired_at(),
        }
    }

    fn to_sqlitedb_model(&self) -> TokenSqliteModel {
        TokenSqliteModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.admin_id,
            &self.token,
            &sqlx::types::Json(
                self.bucket_rules
                    .iter()
                    .map(|(bucket_id, bucket_rules)| (*bucket_id, bucket_rules.to_sqlitedb_model()))
                    .collect(),
            ),
            &sqlx::types::Json(
                self.collection_rules
                    .iter()
                    .map(|(collection_id, collection_rules)| {
                        (*collection_id, collection_rules.to_sqlitedb_model())
                    })
                    .collect(),
            ),
            &self.expired_at,
        )
    }
}

#[derive(Serialize, Clone)]
pub struct TokenCollectionRuleMethod {
    find_one: bool,
    find_many: bool,
    insert: bool,
    update: bool,
    delete: bool,
}

impl TokenCollectionRuleMethod {
    pub fn new(
        find_one: &bool,
        find_many: &bool,
        insert: &bool,
        update: &bool,
        delete: &bool,
    ) -> Self {
        Self {
            find_one: *find_one,
            find_many: *find_many,
            insert: *insert,
            update: *update,
            delete: *delete,
        }
    }

    pub fn find_one(&self) -> &bool {
        &self.find_one
    }

    pub fn find_many(&self) -> &bool {
        &self.find_many
    }

    pub fn insert(&self) -> &bool {
        &self.insert
    }

    pub fn update(&self) -> &bool {
        &self.update
    }

    pub fn delete(&self) -> &bool {
        &self.delete
    }

    pub fn from_scylladb_model(model: &TokenCollectionRuleMethodScyllaModel) -> Self {
        Self {
            find_one: *model.find_one(),
            find_many: *model.find_many(),
            insert: *model.insert(),
            update: *model.update(),
            delete: *model.delete(),
        }
    }

    pub fn to_scylladb_model(&self) -> TokenCollectionRuleMethodScyllaModel {
        TokenCollectionRuleMethodScyllaModel::new(
            &self.find_one,
            &self.find_many,
            &self.insert,
            &self.update,
            &self.delete,
        )
    }

    pub fn from_postgresdb_model(model: &TokenCollectionRuleMethodPostgresModel) -> Self {
        Self {
            find_one: *model.find_one(),
            find_many: *model.find_many(),
            insert: *model.insert(),
            update: *model.update(),
            delete: *model.delete(),
        }
    }

    pub fn to_postgresdb_model(&self) -> TokenCollectionRuleMethodPostgresModel {
        TokenCollectionRuleMethodPostgresModel::new(
            &self.find_one,
            &self.find_many,
            &self.insert,
            &self.update,
            &self.delete,
        )
    }

    pub fn from_mysqldb_model(model: &TokenCollectionRuleMethodMysqlModel) -> Self {
        Self {
            find_one: *model.find_one(),
            find_many: *model.find_many(),
            insert: *model.insert(),
            update: *model.update(),
            delete: *model.delete(),
        }
    }

    pub fn to_mysqldb_model(&self) -> TokenCollectionRuleMethodMysqlModel {
        TokenCollectionRuleMethodMysqlModel::new(
            &self.find_one,
            &self.find_many,
            &self.insert,
            &self.update,
            &self.delete,
        )
    }

    pub fn from_sqlitedb_model(model: &TokenCollectionRuleMethodSqliteModel) -> Self {
        Self {
            find_one: *model.find_one(),
            find_many: *model.find_many(),
            insert: *model.insert(),
            update: *model.update(),
            delete: *model.delete(),
        }
    }

    pub fn to_sqlitedb_model(&self) -> TokenCollectionRuleMethodSqliteModel {
        TokenCollectionRuleMethodSqliteModel::new(
            &self.find_one,
            &self.find_many,
            &self.insert,
            &self.update,
            &self.delete,
        )
    }
}

#[derive(Serialize, Clone)]
pub struct TokenBucketRuleMethod {
    find_one: bool,
    find_many: bool,
    insert: bool,
    update: bool,
    delete: bool,
    download_one: bool,
}

impl TokenBucketRuleMethod {
    pub fn new(
        find_one: &bool,
        find_many: &bool,
        insert: &bool,
        update: &bool,
        delete: &bool,
        download_one: &bool,
    ) -> Self {
        Self {
            find_one: *find_one,
            find_many: *find_many,
            insert: *insert,
            update: *update,
            delete: *delete,
            download_one: *download_one,
        }
    }

    pub fn find_one(&self) -> &bool {
        &self.find_one
    }

    pub fn find_many(&self) -> &bool {
        &self.find_many
    }

    pub fn insert(&self) -> &bool {
        &self.insert
    }

    pub fn update(&self) -> &bool {
        &self.update
    }

    pub fn delete(&self) -> &bool {
        &self.delete
    }

    pub fn download_one(&self) -> &bool {
        &self.download_one
    }

    pub fn from_scylladb_model(model: &TokenBucketRuleMethodScyllaModel) -> Self {
        Self {
            find_one: *model.find_one(),
            find_many: *model.find_many(),
            insert: *model.insert(),
            update: *model.update(),
            delete: *model.delete(),
            download_one: *model.download_one(),
        }
    }

    pub fn to_scylladb_model(&self) -> TokenBucketRuleMethodScyllaModel {
        TokenBucketRuleMethodScyllaModel::new(
            &self.find_one,
            &self.find_many,
            &self.insert,
            &self.update,
            &self.delete,
            &self.download_one,
        )
    }

    pub fn from_postgresdb_model(model: &TokenBucketRuleMethodPostgresModel) -> Self {
        Self {
            find_one: *model.find_one(),
            find_many: *model.find_many(),
            insert: *model.insert(),
            update: *model.update(),
            delete: *model.delete(),
            download_one: *model.download_one(),
        }
    }

    pub fn to_postgresdb_model(&self) -> TokenBucketRuleMethodPostgresModel {
        TokenBucketRuleMethodPostgresModel::new(
            &self.find_one,
            &self.find_many,
            &self.insert,
            &self.update,
            &self.delete,
            &self.download_one,
        )
    }

    pub fn from_mysqldb_model(model: &TokenBucketRuleMethodMysqlModel) -> Self {
        Self {
            find_one: *model.find_one(),
            find_many: *model.find_many(),
            insert: *model.insert(),
            update: *model.update(),
            delete: *model.delete(),
            download_one: *model.download_one(),
        }
    }

    pub fn to_mysqldb_model(&self) -> TokenBucketRuleMethodMysqlModel {
        TokenBucketRuleMethodMysqlModel::new(
            &self.find_one,
            &self.find_many,
            &self.insert,
            &self.update,
            &self.delete,
            &self.download_one,
        )
    }

    pub fn from_sqlitedb_model(model: &TokenBucketRuleMethodSqliteModel) -> Self {
        Self {
            find_one: *model.find_one(),
            find_many: *model.find_many(),
            insert: *model.insert(),
            update: *model.update(),
            delete: *model.delete(),
            download_one: *model.download_one(),
        }
    }

    pub fn to_sqlitedb_model(&self) -> TokenBucketRuleMethodSqliteModel {
        TokenBucketRuleMethodSqliteModel::new(
            &self.find_one,
            &self.find_many,
            &self.insert,
            &self.update,
            &self.delete,
            &self.download_one,
        )
    }
}
