use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use hb_db_mysql::model::token::TokenModel as TokenMysqlModel;
use hb_db_postgresql::model::token::TokenModel as TokenPostgresModel;
use hb_db_scylladb::model::token::TokenModel as TokenScyllaModel;
use hb_db_sqlite::model::token::TokenModel as TokenSqliteModel;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    bucket_rule::{BucketPermission, BucketRuleDao},
    collection_rule::{CollectionPermission, CollectionRuleDao},
    util::conversion,
    Db,
};

#[derive(Deserialize, Serialize)]
pub struct TokenDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    project_id: Uuid,
    admin_id: Uuid,
    name: String,
    token: String,
    allow_anonymous: bool,
    expired_at: Option<DateTime<Utc>>,
}

impl TokenDao {
    pub fn new(
        project_id: &Uuid,
        admin_id: &Uuid,
        name: &str,
        token_length: &usize,
        allow_anonymous: &bool,
        expired_at: &Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            created_at: now,
            updated_at: now,
            project_id: *project_id,
            admin_id: *admin_id,
            name: name.to_owned(),
            token: thread_rng()
                .sample_iter(&Alphanumeric)
                .take(*token_length)
                .map(char::from)
                .collect(),
            allow_anonymous: *allow_anonymous,
            expired_at: *expired_at,
        }
    }

    pub fn from_bytes<'a>(bytes: &'a [u8]) -> Result<Self, rmp_serde::decode::Error>
    where
        Self: Deserialize<'a>,
    {
        rmp_serde::from_slice(bytes)
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(self)
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

    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn allow_anonymous(&self) -> &bool {
        &self.allow_anonymous
    }

    pub fn expired_at(&self) -> &Option<DateTime<Utc>> {
        &self.expired_at
    }

    pub fn set_admin_id(&mut self, admin_id: &Uuid) {
        self.admin_id = *admin_id;
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

    pub fn set_allow_anonymous(&mut self, allow_anonymous: &bool) {
        self.allow_anonymous = *allow_anonymous;
    }

    pub fn set_expired_at(&mut self, expired_at: &Option<DateTime<Utc>>) {
        self.expired_at = *expired_at;
    }

    pub async fn is_allow_find_one_file(
        &self,
        db: &Db,
        bucket_id: &Uuid,
    ) -> Option<BucketPermission> {
        if let Ok(bucket_rule_data) =
            BucketRuleDao::db_select_by_token_id_and_bucket_id(db, &self.id, bucket_id).await
        {
            Some(*bucket_rule_data.find_one())
        } else {
            None
        }
    }

    pub async fn is_allow_find_many_files(
        &self,
        db: &Db,
        bucket_id: &Uuid,
    ) -> Option<BucketPermission> {
        if let Ok(bucket_rule_data) =
            BucketRuleDao::db_select_by_token_id_and_bucket_id(db, &self.id, bucket_id).await
        {
            Some(*bucket_rule_data.find_many())
        } else {
            None
        }
    }

    pub async fn is_allow_insert_file(&self, db: &Db, bucket_id: &Uuid) -> bool {
        if let Ok(bucket_rule_data) =
            BucketRuleDao::db_select_by_token_id_and_bucket_id(db, &self.id, bucket_id).await
        {
            *bucket_rule_data.insert_one()
        } else {
            false
        }
    }

    pub async fn is_allow_update_file(
        &self,
        db: &Db,
        bucket_id: &Uuid,
    ) -> Option<BucketPermission> {
        if let Ok(bucket_rule_data) =
            BucketRuleDao::db_select_by_token_id_and_bucket_id(db, &self.id, bucket_id).await
        {
            Some(*bucket_rule_data.update_one())
        } else {
            None
        }
    }

    pub async fn is_allow_delete_file(
        &self,
        db: &Db,
        bucket_id: &Uuid,
    ) -> Option<BucketPermission> {
        if let Ok(bucket_rule_data) =
            BucketRuleDao::db_select_by_token_id_and_bucket_id(db, &self.id, bucket_id).await
        {
            Some(*bucket_rule_data.update_one())
        } else {
            None
        }
    }

    pub async fn is_allow_find_one_record(
        &self,
        db: &Db,
        collection_id: &Uuid,
    ) -> Option<CollectionPermission> {
        if let Ok(collection_rule_data) =
            CollectionRuleDao::db_select_by_token_id_and_collection_id(db, &self.id, collection_id)
                .await
        {
            Some(*collection_rule_data.find_one())
        } else {
            None
        }
    }

    pub async fn is_allow_find_many_records(
        &self,
        db: &Db,
        collection_id: &Uuid,
    ) -> Option<CollectionPermission> {
        if let Ok(collection_rule_data) =
            CollectionRuleDao::db_select_by_token_id_and_collection_id(db, &self.id, collection_id)
                .await
        {
            Some(*collection_rule_data.find_many())
        } else {
            None
        }
    }

    pub async fn is_allow_insert_record(&self, db: &Db, collection_id: &Uuid) -> bool {
        if let Ok(collection_rule_data) =
            CollectionRuleDao::db_select_by_token_id_and_collection_id(db, &self.id, collection_id)
                .await
        {
            *collection_rule_data.insert_one()
        } else {
            false
        }
    }

    pub async fn is_allow_update_record(
        &self,
        db: &Db,
        collection_id: &Uuid,
    ) -> Option<CollectionPermission> {
        if let Ok(collection_rule_data) =
            CollectionRuleDao::db_select_by_token_id_and_collection_id(db, &self.id, collection_id)
                .await
        {
            Some(*collection_rule_data.update_one())
        } else {
            None
        }
    }

    pub async fn is_allow_delete_record(
        &self,
        db: &Db,
        collection_id: &Uuid,
    ) -> Option<CollectionPermission> {
        if let Ok(collection_rule_data) =
            CollectionRuleDao::db_select_by_token_id_and_collection_id(db, &self.id, collection_id)
                .await
        {
            Some(*collection_rule_data.delete_one())
        } else {
            None
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

    pub async fn db_select_many_by_admin_id_and_project_id(
        db: &Db,
        admin_id: &Uuid,
        project_id: &Uuid,
    ) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                let mut tokens_data = Vec::new();
                for token in db
                    .select_many_tokens_by_admin_id_and_project_id(admin_id, project_id)
                    .await?
                {
                    tokens_data.push(Self::from_scylladb_model(&token?)?);
                }
                Ok(tokens_data)
            }
            Db::PostgresqlDb(db) => Ok(db
                .select_many_tokens_by_admin_id_and_project_id(admin_id, project_id)
                .await?
                .iter()
                .map(|data| Self::from_postgresdb_model(data))
                .collect()),
            Db::MysqlDb(db) => Ok(db
                .select_many_tokens_by_admin_id_and_project_id(admin_id, project_id)
                .await?
                .iter()
                .map(|data| Self::from_mysqldb_model(data))
                .collect()),
            Db::SqliteDb(db) => Ok(db
                .select_many_tokens_by_admin_id_and_project_id(admin_id, project_id)
                .await?
                .iter()
                .map(|data| Self::from_sqlitedb_model(data))
                .collect()),
        }
    }

    pub async fn db_select_many_by_project_id(db: &Db, project_id: &Uuid) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                let mut tokens_data = Vec::new();
                for token in db.select_many_tokens_by_project_id(project_id).await? {
                    tokens_data.push(Self::from_scylladb_model(&token?)?);
                }
                Ok(tokens_data)
            }
            Db::PostgresqlDb(db) => Ok(db
                .select_many_tokens_by_project_id(project_id)
                .await?
                .iter()
                .map(|data| Self::from_postgresdb_model(data))
                .collect()),
            Db::MysqlDb(db) => Ok(db
                .select_many_tokens_by_project_id(project_id)
                .await?
                .iter()
                .map(|data| Self::from_mysqldb_model(data))
                .collect()),
            Db::SqliteDb(db) => Ok(db
                .select_many_tokens_by_project_id(project_id)
                .await?
                .iter()
                .map(|data| Self::from_sqlitedb_model(data))
                .collect()),
        }
    }

    pub async fn db_select_many_from_updated_at_and_after_id_with_limit_asc(
        db: &Db,
        updated_at: &DateTime<Utc>,
        id: &Uuid,
        limit: &i32,
    ) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(_) => Err(Error::msg("Unimplemented")),
            Db::PostgresqlDb(db) => {
                let tokens = db
                    .select_many_tokens_from_updated_at_and_after_id_with_limit_asc(
                        updated_at, id, limit,
                    )
                    .await?;
                let mut tokens_data = Vec::with_capacity(tokens.len());
                for token in &tokens {
                    tokens_data.push(Self::from_postgresdb_model(token));
                }
                Ok(tokens_data)
            }
            Db::MysqlDb(db) => {
                let tokens = db
                    .select_many_tokens_from_updated_at_and_after_id_with_limit_asc(
                        updated_at, id, limit,
                    )
                    .await?;
                let mut tokens_data = Vec::with_capacity(tokens.len());
                for token in &tokens {
                    tokens_data.push(Self::from_mysqldb_model(token));
                }
                Ok(tokens_data)
            }
            Db::SqliteDb(db) => {
                let tokens = db
                    .select_many_tokens_from_updated_at_and_after_id_with_limit_asc(
                        updated_at, id, limit,
                    )
                    .await?;
                let mut tokens_data = Vec::with_capacity(tokens.len());
                for token in &tokens {
                    tokens_data.push(Self::from_sqlitedb_model(token));
                }
                Ok(tokens_data)
            }
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
        tokio::try_join!(
            CollectionRuleDao::db_delete_many_by_token_id(db, id),
            BucketRuleDao::db_delete_many_by_token_id(db, id)
        )?;

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
            project_id: *model.project_id(),
            admin_id: *model.admin_id(),
            name: model.name().to_owned(),
            token: model.token().to_owned(),
            allow_anonymous: *model.allow_anonymous(),
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
            &conversion::datetime_utc_to_scylla_cql_timestamp(&self.created_at),
            &conversion::datetime_utc_to_scylla_cql_timestamp(&self.updated_at),
            &self.project_id,
            &self.admin_id,
            &self.name,
            &self.token,
            &self.allow_anonymous,
            &match &self.expired_at {
                Some(expired_at) => {
                    Some(conversion::datetime_utc_to_scylla_cql_timestamp(expired_at))
                }
                None => None,
            },
        )
    }

    fn from_postgresdb_model(model: &TokenPostgresModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            project_id: *model.project_id(),
            admin_id: *model.admin_id(),
            name: model.name().to_owned(),
            token: model.token().to_owned(),
            allow_anonymous: *model.allow_anonymous(),
            expired_at: *model.expired_at(),
        }
    }

    fn to_postgresdb_model(&self) -> TokenPostgresModel {
        TokenPostgresModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.project_id,
            &self.admin_id,
            &self.name,
            &self.token,
            &self.allow_anonymous,
            &self.expired_at,
        )
    }

    fn from_mysqldb_model(model: &TokenMysqlModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            project_id: *model.project_id(),
            admin_id: *model.admin_id(),
            name: model.name().to_owned(),
            token: model.token().to_owned(),
            allow_anonymous: *model.allow_anonymous(),
            expired_at: *model.expired_at(),
        }
    }

    fn to_mysqldb_model(&self) -> TokenMysqlModel {
        TokenMysqlModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.project_id,
            &self.admin_id,
            &self.name,
            &self.token,
            &self.allow_anonymous,
            &self.expired_at,
        )
    }

    fn from_sqlitedb_model(model: &TokenSqliteModel) -> Self {
        Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            project_id: *model.project_id(),
            admin_id: *model.admin_id(),
            name: model.name().to_owned(),
            token: model.token().to_owned(),
            allow_anonymous: *model.allow_anonymous(),
            expired_at: *model.expired_at(),
        }
    }

    fn to_sqlitedb_model(&self) -> TokenSqliteModel {
        TokenSqliteModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.project_id,
            &self.admin_id,
            &self.name,
            &self.token,
            &self.allow_anonymous,
            &self.expired_at,
        )
    }
}
