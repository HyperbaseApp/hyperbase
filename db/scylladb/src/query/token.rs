use anyhow::Result;
use scylla::{transport::session::TypedRowIter, CachingSession};
use uuid::Uuid;

use crate::{db::ScyllaDb, model::token::TokenModel};

const INSERT: &str = "INSERT INTO \"hyperbase\".\"tokens\" (\"id\", \"created_at\", \"updated_at\", \"project_id\", \"admin_id\", \"name\", \"token\", \"allow_anonymous\", \"expired_at\") VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"admin_id\", \"name\", \"token\", \"allow_anonymous\", \"expired_at\" FROM \"hyperbase\".\"tokens\" WHERE \"id\" = ?";
const SELECT_MANY_BY_ADMIN_ID_AND_PROJECT_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"admin_id\", \"name\", \"token\", \"allow_anonymous\", \"expired_at\" FROM \"hyperbase\".\"tokens\" WHERE \"admin_id\" = ? AND \"project_id\" = ? ORDER BY \"id\" DESC ALLOW FILTERING";
const SELECT_MANY_BY_PROJECT_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"admin_id\", \"name\", \"token\", \"allow_anonymous\", \"expired_at\" FROM \"hyperbase\".\"tokens\" WHERE \"project_id\" = ? ORDER BY \"id\" DESC";
const UPDATE: &str = "UPDATE \"hyperbase\".\"tokens\" SET \"updated_at\" = ?, \"admin_id\" = ?, \"name\" = ?, \"allow_anonymous\" = ?, \"expired_at\" = ? WHERE \"id\" = ?";
const DELETE: &str = "DELETE FROM \"hyperbase\".\"tokens\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession) {
    hb_log::info(Some("🔧"), "ScyllaDB: Setting up tokens table");

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"tokens\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"project_id\" uuid, \"admin_id\" uuid, \"name\" text, \"token\" text, \"allow_anonymous\" boolean, \"expired_at\" timestamp, PRIMARY KEY (\"id\"))", &[]).await.unwrap();
    cached_session
        .get_session()
        .query(
            "CREATE INDEX IF NOT EXISTS ON \"hyperbase\".\"tokens\" (\"admin_id\")",
            &[],
        )
        .await
        .unwrap();
    cached_session
        .get_session()
        .query(
            "CREATE INDEX IF NOT EXISTS ON \"hyperbase\".\"tokens\" (\"project_id\")",
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
        .add_prepared_statement(&SELECT_MANY_BY_ADMIN_ID_AND_PROJECT_ID.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT_MANY_BY_PROJECT_ID.into())
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
    pub async fn insert_token(&self, value: &TokenModel) -> Result<()> {
        self.execute(INSERT, value).await?;
        Ok(())
    }

    pub async fn select_token(&self, id: &Uuid) -> Result<TokenModel> {
        Ok(self
            .execute(SELECT, [id].as_ref())
            .await?
            .first_row_typed()?)
    }

    pub async fn select_many_tokens_by_admin_id_and_project_id(
        &self,
        admin_id: &Uuid,
        project_id: &Uuid,
    ) -> Result<TypedRowIter<TokenModel>> {
        Ok(self
            .execute(
                SELECT_MANY_BY_ADMIN_ID_AND_PROJECT_ID,
                [admin_id, project_id].as_ref(),
            )
            .await?
            .rows_typed()?)
    }

    pub async fn select_many_tokens_by_project_id(
        &self,
        project_id: &Uuid,
    ) -> Result<TypedRowIter<TokenModel>> {
        Ok(self
            .execute(SELECT_MANY_BY_PROJECT_ID, [project_id].as_ref())
            .await?
            .rows_typed()?)
    }

    pub async fn update_token(&self, value: &TokenModel) -> Result<()> {
        self.execute(
            UPDATE,
            &(
                value.updated_at(),
                value.admin_id(),
                value.name(),
                value.allow_anonymous(),
                value.expired_at(),
                value.id(),
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_token(&self, id: &Uuid) -> Result<()> {
        self.execute(DELETE, [id].as_ref()).await?;
        Ok(())
    }
}
