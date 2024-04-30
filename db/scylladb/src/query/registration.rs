use anyhow::Result;
use scylla::CachingSession;
use uuid::Uuid;

use crate::{db::ScyllaDb, model::registration::RegistrationModel};

const INSERT: &str = "INSERT INTO \"hyperbase\".\"registrations\" (\"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\", \"code\") VALUES (?, ?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\", \"code\" FROM \"hyperbase\".\"registrations\" WHERE \"id\" = ?";
const SELECT_BY_EMAIL: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\", \"code\" FROM \"hyperbase\".\"registrations\" WHERE \"email\" = ?";
const UPDATE: &str = "UPDATE \"hyperbase\".\"registrations\" SET \"updated_at\" = ?, \"code\" = ? WHERE \"id\" = ?";
const DELETE: &str = "DELETE FROM \"hyperbase\".\"registrations\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession, ttl: &u32) {
    hb_log::info(Some("🔧"), "[ScyllaDB] Setting up registrations table");

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"registrations\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"email\" text, \"password_hash\" text, \"code\" text, PRIMARY KEY (\"id\")) WITH default_time_to_live = ".to_owned() + &ttl.to_string(), &[]).await.unwrap();
    cached_session
        .get_session()
        .query(
            "CREATE INDEX IF NOT EXISTS ON \"hyperbase\".\"registrations\" (\"email\")",
            &[],
        )
        .await
        .unwrap();

    cached_session
        .add_prepared_statement(&INSERT.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT_BY_EMAIL.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&UPDATE.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&DELETE.into())
        .await
        .unwrap();
}

impl ScyllaDb {
    pub async fn insert_registration(&self, value: &RegistrationModel) -> Result<()> {
        self.execute(INSERT, value).await?;
        Ok(())
    }

    pub async fn select_registration(&self, id: &Uuid) -> Result<RegistrationModel> {
        Ok(self
            .execute(SELECT, [id].as_ref())
            .await?
            .first_row_typed()?)
    }

    pub async fn select_registration_by_email(&self, email: &str) -> Result<RegistrationModel> {
        Ok(self
            .execute(SELECT_BY_EMAIL, [email].as_ref())
            .await?
            .first_row_typed()?)
    }

    pub async fn update_registration(&self, value: &RegistrationModel) -> Result<()> {
        self.execute(UPDATE, &(value.updated_at(), value.code(), value.id()))
            .await?;
        Ok(())
    }

    pub async fn delete_registration(&self, id: &Uuid) -> Result<()> {
        self.execute(DELETE, [id].as_ref()).await?;
        Ok(())
    }
}
