use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_scylladb::{db::ScyllaDb, model::token::TokenScyllaModel};
use scylla::frame::value::Timestamp;
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
    expired_at: DateTime<Utc>,
}

impl TokenDao {
    pub fn new(admin_id: &Uuid, token: &str, expired_at: &DateTime<Utc>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            admin_id: *admin_id,
            token: token.to_owned(),
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

    pub fn expired_at(&self) -> &DateTime<Utc> {
        &self.expired_at
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_insert(&self, db).await,
        }
    }

    pub async fn db_select(db: &Db, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Ok(Self::from_scylladb_model(
                &Self::scylladb_select(db, id).await?,
            )?),
        }
    }

    pub async fn db_select_by_token(db: &Db, token: &str) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Ok(Self::from_scylladb_model(
                &Self::scylladb_select_by_token(db, token).await?,
            )?),
        }
    }

    async fn scylladb_insert(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(
            db.prepared_statement().token().insert(),
            self.to_scylladb_model(),
        )
        .await?;
        Ok(())
    }

    async fn scylladb_select(db: &ScyllaDb, id: &Uuid) -> Result<TokenScyllaModel> {
        Ok(db
            .execute(db.prepared_statement().token().select(), [id].as_ref())
            .await?
            .first_row_typed::<TokenScyllaModel>()?)
    }

    async fn scylladb_select_by_token(db: &ScyllaDb, token: &str) -> Result<TokenScyllaModel> {
        Ok(db
            .execute(
                db.prepared_statement().token().select_by_token(),
                [token].as_ref(),
            )
            .await?
            .first_row_typed::<TokenScyllaModel>()?)
    }

    fn from_scylladb_model(model: &TokenScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: duration_since_epoch_to_datetime(model.created_at().0)?,
            updated_at: duration_since_epoch_to_datetime(model.updated_at().0)?,
            admin_id: *model.admin_id(),
            token: model.token().to_owned(),
            expired_at: duration_since_epoch_to_datetime(model.expired_at().0)?,
        })
    }

    fn to_scylladb_model(&self) -> TokenScyllaModel {
        TokenScyllaModel::new(
            &self.id,
            &Timestamp(datetime_to_duration_since_epoch(self.created_at)),
            &Timestamp(datetime_to_duration_since_epoch(self.updated_at)),
            &self.admin_id,
            &self.token,
            &Timestamp(datetime_to_duration_since_epoch(self.expired_at)),
        )
    }
}
