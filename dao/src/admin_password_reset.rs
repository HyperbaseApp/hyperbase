use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_scylladb::{db::ScyllaDb, model::admin_password_reset::AdminPasswordResetScyllaModel};
use scylla::frame::value::Timestamp;
use uuid::Uuid;

use crate::{
    util::conversion::{datetime_to_duration_since_epoch, duration_since_epoch_to_datetime},
    Db,
};

pub struct AdminPasswordResetDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    email: String,
    code: String,
}

impl AdminPasswordResetDao {
    pub fn new(email: &str, code: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            email: email.to_string(),
            code: code.to_string(),
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

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn code(&self) -> &str {
        &self.code
    }
}

impl AdminPasswordResetDao {
    pub async fn insert(&self, db: &Db<'_>) -> Result<()> {
        match *db {
            Db::ScyllaDb(db) => Self::scylladb_insert(&self, db).await,
        }
    }

    pub async fn select(db: &Db<'_>, id: &Uuid) -> Result<Self> {
        match *db {
            Db::ScyllaDb(db) => Ok(Self::from_scylladb_model(
                &Self::scylladb_select(db, id).await?,
            )?),
        }
    }
}

impl AdminPasswordResetDao {
    async fn scylladb_insert(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(
            db.prepared_statement().admin_password_reset().insert(),
            self.to_scylladb_model(),
        )
        .await?;
        Ok(())
    }

    async fn scylladb_select(db: &ScyllaDb, id: &Uuid) -> Result<AdminPasswordResetScyllaModel> {
        Ok(db
            .execute(
                db.prepared_statement().admin_password_reset().select(),
                [id].as_ref(),
            )
            .await?
            .first_row_typed::<AdminPasswordResetScyllaModel>()?)
    }

    fn from_scylladb_model(model: &AdminPasswordResetScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: duration_since_epoch_to_datetime(model.created_at().0)?,
            updated_at: duration_since_epoch_to_datetime(model.updated_at().0)?,
            email: model.email().to_string(),
            code: model.code().to_string(),
        })
    }

    fn to_scylladb_model(&self) -> AdminPasswordResetScyllaModel {
        AdminPasswordResetScyllaModel::new(
            self.id,
            Timestamp(datetime_to_duration_since_epoch(self.created_at)),
            Timestamp(datetime_to_duration_since_epoch(self.updated_at)),
            self.email.clone(),
            self.code.clone(),
        )
    }
}
