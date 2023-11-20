use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_scylladb::{
    db::ScyllaDb,
    model::admin::{AdminScyllaModel, AdminScyllaRole},
};
use scylla::frame::value::Timestamp;
use strum::EnumString;
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
    role: AdminRole,
}

impl AdminDao {
    pub fn new(email: &str, password_hash: &str, role: &AdminRole) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            role: *role,
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

    pub fn set_email(&mut self, email: &str) {
        self.email = email.to_string()
    }

    pub fn set_password_hash(&mut self, password_hash: &str) {
        self.password_hash = password_hash.to_string();
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

    pub async fn db_select_by_email(db: &Db, email: &str) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Ok(Self::from_scylladb_model(
                &Self::scylladb_select_by_email(db, email).await?,
            )?),
        }
    }

    pub async fn db_update(&mut self, db: &Db) -> Result<()> {
        self.updated_at = Utc::now();
        match db {
            Db::ScyllaDb(db) => Self::scylladb_update(&self, db).await,
        }
    }

    pub async fn db_delete(db: &Db, id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_delete(db, id).await,
        }
    }

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
        db.execute(
            db.prepared_statement().admin().update(),
            (&self.updated_at, &self.email, &self.password_hash, &self.id),
        )
        .await?;
        Ok(())
    }

    async fn scylladb_delete(db: &ScyllaDb, id: &Uuid) -> Result<()> {
        db.execute(db.prepared_statement().admin().delete(), [id].as_ref())
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
            role: AdminRole::from_scylladb_model(model.role()),
        })
    }

    fn to_scylladb_model(&self) -> AdminScyllaModel {
        AdminScyllaModel::new(
            &self.id,
            &Timestamp(datetime_to_duration_since_epoch(self.created_at)),
            &Timestamp(datetime_to_duration_since_epoch(self.updated_at)),
            &self.email,
            &self.password_hash,
            &self.role.to_scylladb_model(),
        )
    }
}

#[derive(EnumString, Clone, Copy)]
pub enum AdminRole {
    SuperUser,
    User,
}

impl AdminRole {
    pub fn from_scylladb_model(model: &AdminScyllaRole) -> Self {
        match model {
            AdminScyllaRole::SuperUser => Self::SuperUser,
            AdminScyllaRole::User => Self::User,
        }
    }

    pub fn to_scylladb_model(&self) -> AdminScyllaRole {
        match self {
            AdminRole::SuperUser => AdminScyllaRole::SuperUser,
            AdminRole::User => AdminScyllaRole::User,
        }
    }
}
