use ahash::HashMap;
use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_mysql::{
    db::MysqlDb,
    model::token::TokenModel as TokenMysqlModel,
    query::token::{
        DELETE as MYSQL_DELETE, INSERT as MYSQL_INSERT, SELECT as MYSQL_SELECT,
        SELECT_BY_TOKEN as MYSQL_SELECT_BY_TOKEN,
        SELECT_MANY_BY_ADMIN_ID as MYSQL_SELECT_MANY_BY_ADMIN_ID, UPDATE as MYSQL_UPDATE,
    },
};
use hb_db_postgresql::{
    db::PostgresDb,
    model::token::TokenModel as TokenPostgresModel,
    query::token::{
        DELETE as POSTGRES_DELETE, INSERT as POSTGRES_INSERT, SELECT as POSTGRES_SELECT,
        SELECT_BY_TOKEN as POSTGRES_SELECT_BY_TOKEN,
        SELECT_MANY_BY_ADMIN_ID as POSTGRES_SELECT_MANY_BY_ADMIN_ID, UPDATE as POSTGRES_UPDATE,
    },
};
use hb_db_scylladb::{
    db::ScyllaDb,
    model::token::TokenModel as TokenScyllaModel,
    query::token::{
        DELETE as SCYLLA_DELETE, INSERT as SCYLLA_INSERT, SELECT as SCYLLA_SELECT,
        SELECT_BY_TOKEN as SCYLLA_SELECT_BY_TOKEN,
        SELECT_MANY_BY_ADMIN_ID as SCYLLA_SELECT_MANY_BY_ADMIN_ID, UPDATE as SCYLLA_UPDATE,
    },
};
use hb_db_sqlite::{
    db::SqliteDb,
    model::token::TokenModel as TokenSqliteModel,
    query::token::{
        DELETE as SQLITE_DELETE, INSERT as SQLITE_INSERT, SELECT as SQLITE_SELECT,
        SELECT_BY_TOKEN as SQLITE_SELECT_BY_TOKEN,
        SELECT_MANY_BY_ADMIN_ID as SQLITE_SELECT_MANY_BY_ADMIN_ID, UPDATE as SQLITE_UPDATE,
    },
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use scylla::{frame::value::Timestamp, transport::session::TypedRowIter};
use uuid::Uuid;

use crate::{
    util::conversion::{datetime_to_duration_since_epoch, duration_since_epoch_to_datetime},
    Db,
};

pub struct TokenDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    admin_id: Uuid,
    token: String,
    rules: HashMap<Uuid, i8>,
    expired_at: Option<DateTime<Utc>>,
}

impl TokenDao {
    pub fn new(
        admin_id: &Uuid,
        token_length: &usize,
        rules: &HashMap<Uuid, i8>,
        expired_at: &Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            admin_id: *admin_id,
            token: thread_rng()
                .sample_iter(&Alphanumeric)
                .take(*token_length)
                .map(char::from)
                .collect(),
            rules: rules.clone(),
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

    pub fn rules(&self) -> &HashMap<Uuid, i8> {
        &self.rules
    }

    pub fn expired_at(&self) -> &Option<DateTime<Utc>> {
        &self.expired_at
    }

    pub fn upsert_rule(&mut self, collection_id: &Uuid, rule: &i8) {
        self.rules.insert(*collection_id, *rule);
    }

    pub fn set_rules(&mut self, rules: &HashMap<Uuid, i8>) {
        self.rules = rules.clone();
    }

    pub fn set_expired_at(&mut self, expired_at: &Option<DateTime<Utc>>) {
        self.expired_at = *expired_at;
    }

    pub fn is_allow_write(&self, collection_id: &Uuid) -> bool {
        self.is_allow(&2, collection_id)
    }

    pub fn is_allow_read(&self, collection_id: &Uuid) -> bool {
        self.is_allow(&1, collection_id)
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_insert(self, db).await,
            Db::PostgresqlDb(db) => Self::postgresdb_insert(self, db).await,
            Db::MysqlDb(db) => Self::mysqldb_insert(self, db).await,
            Db::SqliteDb(db) => Self::sqlitedb_insert(self, db).await,
        }
    }

    pub async fn db_select(db: &Db, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Ok(Self::from_scylladb_model(
                &Self::scylladb_select(db, id).await?,
            )?),
            Db::PostgresqlDb(db) => Ok(Self::from_postgresdb_model(
                &Self::postgresdb_select(db, id).await?,
            )?),
            Db::MysqlDb(db) => Ok(Self::from_mysqldb_model(
                &Self::mysqldb_select(db, id).await?,
            )?),
            Db::SqliteDb(db) => Ok(Self::from_sqlitedb_model(
                &Self::sqlitedb_select(db, id).await?,
            )?),
        }
    }

    pub async fn db_select_by_token(db: &Db, token: &str) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Ok(Self::from_scylladb_model(
                &Self::scylladb_select_by_token(db, token).await?,
            )?),
            Db::PostgresqlDb(db) => Ok(Self::from_postgresdb_model(
                &Self::postgresdb_select_by_token(db, token).await?,
            )?),
            Db::MysqlDb(db) => Ok(Self::from_mysqldb_model(
                &Self::mysqldb_select_by_token(db, token).await?,
            )?),
            Db::SqliteDb(db) => Ok(Self::from_sqlitedb_model(
                &Self::sqlitedb_select_by_token(db, token).await?,
            )?),
        }
    }

    pub async fn db_select_many_by_admin_id(db: &Db, admin_id: &Uuid) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                let mut tokens_data = Vec::new();
                let tokens = Self::scylladb_select_many_by_admin_id(db, admin_id).await?;
                for token in tokens {
                    tokens_data.push(Self::from_scylladb_model(&token?)?);
                }
                Ok(tokens_data)
            }
            Db::PostgresqlDb(db) => {
                let tokens = Self::postgresdb_select_many_by_admin_id(db, admin_id).await?;
                let mut tokens_data = Vec::with_capacity(tokens.len());
                for token in &tokens {
                    tokens_data.push(Self::from_postgresdb_model(token)?);
                }
                Ok(tokens_data)
            }
            Db::MysqlDb(db) => {
                let tokens = Self::mysqldb_select_many_by_admin_id(db, admin_id).await?;
                let mut tokens_data = Vec::with_capacity(tokens.len());
                for token in &tokens {
                    tokens_data.push(Self::from_mysqldb_model(token)?);
                }
                Ok(tokens_data)
            }
            Db::SqliteDb(db) => {
                let tokens = Self::sqlitedb_select_many_by_admin_id(db, admin_id).await?;
                let mut tokens_data = Vec::with_capacity(tokens.len());
                for token in &tokens {
                    tokens_data.push(Self::from_sqlitedb_model(token)?);
                }
                Ok(tokens_data)
            }
        }
    }

    pub async fn db_update(&mut self, db: &Db) -> Result<()> {
        self.updated_at = Utc::now();
        match db {
            Db::ScyllaDb(db) => Self::scylladb_update(self, db).await,
            Db::PostgresqlDb(db) => Self::postgresdb_update(self, db).await,
            Db::MysqlDb(db) => Self::mysqldb_update(self, db).await,
            Db::SqliteDb(db) => Self::sqlitedb_update(self, db).await,
        }
    }

    pub async fn db_delete(db: &Db, id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_delete(db, id).await,
            Db::PostgresqlDb(db) => Self::postgresdb_delete(db, id).await,
            Db::MysqlDb(db) => Self::mysqldb_delete(db, id).await,
            Db::SqliteDb(db) => Self::sqlitedb_delete(db, id).await,
        }
    }

    fn is_allow(&self, rule: &i8, collection_id: &Uuid) -> bool {
        if let Some(collection_rule) = self.rules.get(collection_id) {
            if collection_rule >= rule {
                if let Some(expired_at) = self.expired_at {
                    if expired_at > Utc::now() {
                        return true;
                    }
                } else {
                    return true;
                }
            }
        }
        false
    }

    async fn scylladb_insert(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(SCYLLA_INSERT, &self.to_scylladb_model()).await?;
        Ok(())
    }

    async fn scylladb_select(db: &ScyllaDb, id: &Uuid) -> Result<TokenScyllaModel> {
        Ok(db
            .execute(SCYLLA_SELECT, [id].as_ref())
            .await?
            .first_row_typed::<TokenScyllaModel>()?)
    }

    async fn scylladb_select_by_token(db: &ScyllaDb, token: &str) -> Result<TokenScyllaModel> {
        Ok(db
            .execute(SCYLLA_SELECT_BY_TOKEN, [token].as_ref())
            .await?
            .first_row_typed::<TokenScyllaModel>()?)
    }

    async fn scylladb_select_many_by_admin_id(
        db: &ScyllaDb,
        admin_id: &Uuid,
    ) -> Result<TypedRowIter<TokenScyllaModel>> {
        Ok(db
            .execute(SCYLLA_SELECT_MANY_BY_ADMIN_ID, [admin_id].as_ref())
            .await?
            .rows_typed::<TokenScyllaModel>()?)
    }

    async fn scylladb_update(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(
            SCYLLA_UPDATE,
            &(&self.updated_at, &self.rules, &self.expired_at, &self.id),
        )
        .await?;
        Ok(())
    }

    async fn scylladb_delete(db: &ScyllaDb, id: &Uuid) -> Result<()> {
        db.execute(SCYLLA_DELETE, [id].as_ref()).await?;
        Ok(())
    }

    async fn postgresdb_insert(&self, db: &PostgresDb) -> Result<()> {
        db.execute(
            sqlx::query(POSTGRES_INSERT)
                .bind(&self.id)
                .bind(&self.created_at)
                .bind(&self.updated_at)
                .bind(&self.admin_id)
                .bind(&self.token)
                .bind(&sqlx::types::Json(&self.rules))
                .bind(&self.expired_at),
        )
        .await?;
        Ok(())
    }

    async fn postgresdb_select(db: &PostgresDb, id: &Uuid) -> Result<TokenPostgresModel> {
        Ok(db
            .fetch_one(sqlx::query_as(POSTGRES_SELECT).bind(id))
            .await?)
    }

    async fn postgresdb_select_by_token(
        db: &PostgresDb,
        token: &str,
    ) -> Result<TokenPostgresModel> {
        Ok(db
            .fetch_one(sqlx::query_as(POSTGRES_SELECT_BY_TOKEN).bind(token))
            .await?)
    }

    async fn postgresdb_select_many_by_admin_id(
        db: &PostgresDb,
        admin_id: &Uuid,
    ) -> Result<Vec<TokenPostgresModel>> {
        Ok(db
            .fetch_all(sqlx::query_as(POSTGRES_SELECT_MANY_BY_ADMIN_ID).bind(admin_id))
            .await?)
    }

    async fn postgresdb_update(&self, db: &PostgresDb) -> Result<()> {
        db.execute(
            sqlx::query(POSTGRES_UPDATE)
                .bind(&self.updated_at)
                .bind(&sqlx::types::Json(&self.rules))
                .bind(&self.expired_at)
                .bind(&self.id),
        )
        .await?;
        Ok(())
    }

    async fn postgresdb_delete(db: &PostgresDb, id: &Uuid) -> Result<()> {
        db.execute(sqlx::query(POSTGRES_DELETE).bind(id)).await?;
        Ok(())
    }

    async fn mysqldb_insert(&self, db: &MysqlDb) -> Result<()> {
        db.execute(
            sqlx::query(MYSQL_INSERT)
                .bind(&self.id)
                .bind(&self.created_at)
                .bind(&self.updated_at)
                .bind(&self.admin_id)
                .bind(&self.token)
                .bind(&sqlx::types::Json(&self.rules))
                .bind(&self.expired_at),
        )
        .await?;
        Ok(())
    }

    async fn mysqldb_select(db: &MysqlDb, id: &Uuid) -> Result<TokenMysqlModel> {
        Ok(db.fetch_one(sqlx::query_as(MYSQL_SELECT).bind(id)).await?)
    }

    async fn mysqldb_select_by_token(db: &MysqlDb, token: &str) -> Result<TokenMysqlModel> {
        Ok(db
            .fetch_one(sqlx::query_as(MYSQL_SELECT_BY_TOKEN).bind(token))
            .await?)
    }

    async fn mysqldb_select_many_by_admin_id(
        db: &MysqlDb,
        admin_id: &Uuid,
    ) -> Result<Vec<TokenMysqlModel>> {
        Ok(db
            .fetch_all(sqlx::query_as(MYSQL_SELECT_MANY_BY_ADMIN_ID).bind(admin_id))
            .await?)
    }

    async fn mysqldb_update(&self, db: &MysqlDb) -> Result<()> {
        db.execute(
            sqlx::query(MYSQL_UPDATE)
                .bind(&self.updated_at)
                .bind(&sqlx::types::Json(&self.rules))
                .bind(&self.expired_at)
                .bind(&self.id),
        )
        .await?;
        Ok(())
    }

    async fn mysqldb_delete(db: &MysqlDb, id: &Uuid) -> Result<()> {
        db.execute(sqlx::query(MYSQL_DELETE).bind(id)).await?;
        Ok(())
    }

    async fn sqlitedb_insert(&self, db: &SqliteDb) -> Result<()> {
        db.execute(
            sqlx::query(SQLITE_INSERT)
                .bind(&self.id)
                .bind(&self.created_at)
                .bind(&self.updated_at)
                .bind(&self.admin_id)
                .bind(&self.token)
                .bind(&sqlx::types::Json(&self.rules))
                .bind(&self.expired_at),
        )
        .await?;
        Ok(())
    }

    async fn sqlitedb_select(db: &SqliteDb, id: &Uuid) -> Result<TokenSqliteModel> {
        Ok(db.fetch_one(sqlx::query_as(SQLITE_SELECT).bind(id)).await?)
    }

    async fn sqlitedb_select_by_token(db: &SqliteDb, token: &str) -> Result<TokenSqliteModel> {
        Ok(db
            .fetch_one(sqlx::query_as(SQLITE_SELECT_BY_TOKEN).bind(token))
            .await?)
    }

    async fn sqlitedb_select_many_by_admin_id(
        db: &SqliteDb,
        admin_id: &Uuid,
    ) -> Result<Vec<TokenSqliteModel>> {
        Ok(db
            .fetch_all(sqlx::query_as(SQLITE_SELECT_MANY_BY_ADMIN_ID).bind(admin_id))
            .await?)
    }

    async fn sqlitedb_update(&self, db: &SqliteDb) -> Result<()> {
        db.execute(
            sqlx::query(SQLITE_UPDATE)
                .bind(&self.updated_at)
                .bind(&sqlx::types::Json(&self.rules))
                .bind(&self.expired_at)
                .bind(&self.id),
        )
        .await?;
        Ok(())
    }

    async fn sqlitedb_delete(db: &SqliteDb, id: &Uuid) -> Result<()> {
        db.execute(sqlx::query(SQLITE_DELETE).bind(id)).await?;
        Ok(())
    }

    fn from_scylladb_model(model: &TokenScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: duration_since_epoch_to_datetime(&model.created_at().0)?,
            updated_at: duration_since_epoch_to_datetime(&model.updated_at().0)?,
            admin_id: *model.admin_id(),
            token: model.token().to_owned(),
            rules: model.rules().clone(),
            expired_at: match &model.expired_at() {
                Some(expired_at) => Some(duration_since_epoch_to_datetime(&expired_at.0)?),
                None => None,
            },
        })
    }

    fn to_scylladb_model(&self) -> TokenScyllaModel {
        TokenScyllaModel::new(
            &self.id,
            &Timestamp(datetime_to_duration_since_epoch(&self.created_at)),
            &Timestamp(datetime_to_duration_since_epoch(&self.updated_at)),
            &self.admin_id,
            &self.token,
            &self.rules,
            &match &self.expired_at {
                Some(expired_at) => Some(Timestamp(datetime_to_duration_since_epoch(expired_at))),
                None => None,
            },
        )
    }

    fn from_postgresdb_model(model: &TokenPostgresModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            token: model.token().to_owned(),
            rules: model.rules().clone(),
            expired_at: *model.expired_at(),
        })
    }

    fn from_mysqldb_model(model: &TokenMysqlModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            token: model.token().to_owned(),
            rules: model.rules().clone(),
            expired_at: *model.expired_at(),
        })
    }

    fn from_sqlitedb_model(model: &TokenSqliteModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            token: model.token().to_owned(),
            rules: model.rules().clone(),
            expired_at: *model.expired_at(),
        })
    }
}
