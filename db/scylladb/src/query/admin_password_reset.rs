use anyhow::Result;
use scylla::CachingSession;
use uuid::Uuid;

use crate::{db::ScyllaDb, model::admin_password_reset::AdminPasswordResetModel};

const INSERT: &str = "INSERT INTO \"hyperbase\".\"admin_password_resets\" (\"id\", \"created_at\", \"updated_at\", \"admin_id\", \"code\") VALUES (?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"code\" FROM \"hyperbase\".\"admin_password_resets\" WHERE \"id\" = ?";
const UPDATE: &str = "UPDATE \"hyperbase\".\"admin_password_resets\" SET \"updated_at\" = ?, \"code\" = ? WHERE \"id\" = ?";
const DELETE: &str = "DELETE FROM \"hyperbase\".\"admin_password_resets\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession, ttl: &u32) {
    hb_log::info(
        Some("🔧"),
        "[ScyllaDB] Setting up admin_password_resets table",
    );

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"admin_password_resets\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"admin_id\" uuid, \"code\" text, PRIMARY KEY (\"id\")) WITH default_time_to_live = ".to_owned() + &ttl.to_string(), &[]).await.unwrap();

    cached_session
        .add_prepared_statement(&INSERT.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT.into())
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
    pub async fn insert_admin_password_reset(&self, value: &AdminPasswordResetModel) -> Result<()> {
        self.execute(INSERT, value).await?;
        Ok(())
    }

    pub async fn select_admin_password_reset(&self, id: &Uuid) -> Result<AdminPasswordResetModel> {
        Ok(self
            .execute(SELECT, [id].as_ref())
            .await?
            .first_row_typed()?)
    }

    pub async fn update_admin_password_reset(&self, value: &AdminPasswordResetModel) -> Result<()> {
        self.execute(UPDATE, &(value.updated_at(), value.code(), value.id()))
            .await?;
        Ok(())
    }

    pub async fn delete_admin_password_reset(&self, id: &Uuid) -> Result<()> {
        self.execute(DELETE, [id].as_ref()).await?;
        Ok(())
    }
}
