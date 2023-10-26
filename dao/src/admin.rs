use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_scylladb::{db::ScyllaDb, model::admin::AdminScyllaModel};
use scylla::frame::value::Timestamp;
use uuid::Uuid;

use crate::{
    util::conversion::{datetime_to_duration_since_epoch, duration_since_epoch_to_datetime},
    Db,
};

pub struct AdminDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    email: String,
    password_hash: String,
}

impl AdminDao {
    pub fn new(email: &str, password_hash: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            email: email.to_string(),
            password_hash: password_hash.to_string(),
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

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn set_password_hash(&mut self, password_hash: &str) {
        self.password_hash = password_hash.to_string();
    }
}

impl AdminDao {
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

    pub async fn select_by_email(db: &Db<'_>, email: &str) -> Result<Self> {
        match *db {
            Db::ScyllaDb(db) => Ok(Self::from_scylladb_model(
                &Self::scylladb_select_by_email(db, email).await?,
            )?),
        }
    }

    pub async fn update(&mut self, db: &Db<'_>) -> Result<()> {
        match *db {
            Db::ScyllaDb(db) => {
                self.updated_at = Utc::now();
                Self::scylladb_update(&self, db).await
            }
        }
    }
}

impl AdminDao {
    async fn scylladb_insert(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(
            db.prepared_statement().admin().insert(),
            self.to_scylladb_model(),
        )
        .await?;
        Ok(())
    }

    async fn scylladb_select(db: &ScyllaDb, id: &Uuid) -> Result<AdminScyllaModel> {
        Ok(db
            .execute(db.prepared_statement().admin().select(), [id].as_ref())
            .await?
            .first_row_typed::<AdminScyllaModel>()?)
    }

    async fn scylladb_select_by_email(db: &ScyllaDb, email: &str) -> Result<AdminScyllaModel> {
        Ok(db
            .execute(
                db.prepared_statement().admin().select_by_email(),
                [email].as_ref(),
            )
            .await?
            .first_row_typed::<AdminScyllaModel>()?)
    }

    async fn scylladb_update(&self, db: &ScyllaDb) -> Result<()> {
        let data = self.to_scylladb_model();
        db.execute(
            db.prepared_statement().admin().update(),
            (
                data.updated_at(),
                data.email(),
                data.password_hash(),
                data.id(),
            ),
        )
        .await?;
        Ok(())
    }

    fn from_scylladb_model(model: &AdminScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: duration_since_epoch_to_datetime(model.created_at().0)?,
            updated_at: duration_since_epoch_to_datetime(model.updated_at().0)?,
            email: model.email().to_string(),
            password_hash: model.password_hash().to_string(),
        })
    }

    fn to_scylladb_model(&self) -> AdminScyllaModel {
        AdminScyllaModel::new(
            &self.id,
            &Timestamp(datetime_to_duration_since_epoch(self.created_at)),
            &Timestamp(datetime_to_duration_since_epoch(self.updated_at)),
            &self.email,
            &self.password_hash,
        )
    }
}
