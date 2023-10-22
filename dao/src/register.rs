use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_scylladb::{db::ScyllaDb, model::registration::RegistrationScyllaModel};
use rand::Rng;
use scylla::frame::value::Timestamp;
use uuid::Uuid;

use crate::{util::conversion::datetime_to_duration_since_epoch, Db};

pub struct RegistrationDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    email: String,
    password_hash: String,
    code: String,
}

impl RegistrationDao {
    pub fn new(email: String, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            email,
            password_hash,
            code: rand::thread_rng().gen_range(100000..=999999).to_string(),
        }
    }

    pub async fn insert(&self, db: Db<'_>) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_insert(&self, db).await,
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

    fn to_scylladb_model(&self) -> RegistrationScyllaModel {
        RegistrationScyllaModel::new(
            self.id,
            Timestamp(datetime_to_duration_since_epoch(self.created_at)),
            Timestamp(datetime_to_duration_since_epoch(self.updated_at)),
            self.email.clone(),
            self.password_hash.clone(),
            self.code.clone(),
        )
    }
}
