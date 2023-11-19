use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_scylladb::{db::ScyllaDb, model::registration::RegistrationScyllaModel};
use rand::Rng;
use scylla::frame::value::Timestamp;
use uuid::Uuid;

use crate::{
    admin::AdminRole,
    util::conversion::{datetime_to_duration_since_epoch, duration_since_epoch_to_datetime},
    Db,
};

pub struct RegistrationDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    email: String,
    password_hash: String,
    code: String,
    role: AdminRole,
}

impl RegistrationDao {
    pub fn new(email: &str, password_hash: &str, role: &AdminRole) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            code: rand::thread_rng().gen_range(100000..=999999).to_string(),
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

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn role(&self) -> &AdminRole {
        &self.role
    }
}

impl RegistrationDao {
    pub async fn insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_insert(self, db).await,
        }
    }

    pub async fn select(db: &Db, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Ok(Self::from_scylladb_model(
                &Self::scylladb_select(db, id).await?,
            )?),
        }
    }

    pub async fn delete(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_delete(self, db).await,
        }
    }
}

impl RegistrationDao {
    async fn scylladb_insert(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(
            db.prepared_statement().registration().insert(),
            self.to_scylladb_model(),
        )
        .await?;
        Ok(())
    }

    async fn scylladb_select(db: &ScyllaDb, id: &Uuid) -> Result<RegistrationScyllaModel> {
        Ok(db
            .execute(
                db.prepared_statement().registration().select(),
                [id].as_ref(),
            )
            .await?
            .first_row_typed::<RegistrationScyllaModel>()?)
    }

    async fn scylladb_delete(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(
            db.prepared_statement().registration().delete(),
            [self.id()].as_ref(),
        )
        .await?;
        Ok(())
    }

    fn from_scylladb_model(model: &RegistrationScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: duration_since_epoch_to_datetime(model.created_at().0)?,
            updated_at: duration_since_epoch_to_datetime(model.updated_at().0)?,
            email: model.email().to_string(),
            password_hash: model.password_hash().to_string(),
            code: model.code().to_string(),
            role: AdminRole::from_scylladb_model(model.role()),
        })
    }

    fn to_scylladb_model(&self) -> RegistrationScyllaModel {
        RegistrationScyllaModel::new(
            &self.id,
            &Timestamp(datetime_to_duration_since_epoch(self.created_at)),
            &Timestamp(datetime_to_duration_since_epoch(self.updated_at)),
            &self.email,
            &self.password_hash,
            &self.code,
            &self.role.to_scylladb_model(),
        )
    }
}
