use anyhow::Result;
use scylla::{transport::session::TypedRowIter, CachingSession};
use uuid::Uuid;

use crate::{db::ScyllaDb, model::collection::CollectionModel};

const INSERT: &str = "INSERT INTO \"hyperbase\".\"collections\" (\"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"schema_fields\", \"opt_auth_column_id\") VALUES (?, ?, ?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"schema_fields\", \"opt_auth_column_id\" FROM \"hyperbase\".\"collections\" WHERE \"id\" = ?";
const SELECT_MANY_BY_PROJECT_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"schema_fields\", \"opt_auth_column_id\" FROM \"hyperbase\".\"collections\" WHERE \"project_id\" = ?";
const UPDATE: &str = "UPDATE \"hyperbase\".\"collections\" SET \"updated_at\" = ?, \"name\" = ?, \"schema_fields\" = ?, \"opt_auth_column_id\" = ? WHERE \"id\" = ?";
const DELETE: &str = "DELETE FROM \"hyperbase\".\"collections\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession) {
    hb_log::info(Some("🔧"), "ScyllaDB: Setting up collections table");

    cached_session
        .get_session()
        .query(
            "CREATE TYPE IF NOT EXISTS \"hyperbase\".\"schema_field_props\" (\"kind\" text, \"internal_kind\" text, \"required\" boolean, \"unique\" boolean, \"indexed\" boolean, \"auth_column\" boolean)",
            &[],
        )
        .await
        .unwrap();
    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"collections\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"project_id\" uuid, \"name\" text, \"schema_fields\" map<text, frozen<schema_field_props>>, \"opt_auth_column_id\" boolean, PRIMARY KEY (\"id\"))", &[]).await.unwrap();
    cached_session
        .get_session()
        .query(
            "CREATE INDEX IF NOT EXISTS ON \"hyperbase\".\"collections\" (\"project_id\")",
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
    pub async fn insert_collection(&self, value: &CollectionModel) -> Result<()> {
        self.execute(INSERT, value).await?;
        Ok(())
    }

    pub async fn select_collection(&self, id: &Uuid) -> Result<CollectionModel> {
        Ok(self
            .execute(SELECT, [id].as_ref())
            .await?
            .first_row_typed()?)
    }

    pub async fn select_many_collections_by_project_id(
        &self,
        project_id: &Uuid,
    ) -> Result<TypedRowIter<CollectionModel>> {
        Ok(self
            .execute(SELECT_MANY_BY_PROJECT_ID, [project_id].as_ref())
            .await?
            .rows_typed()?)
    }

    pub async fn update_collection(&self, value: &CollectionModel) -> Result<()> {
        self.execute(
            UPDATE,
            &(
                value.updated_at(),
                value.name(),
                value.schema_fields(),
                value.opt_auth_column_id(),
                value.id(),
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_collection(&self, id: &Uuid) -> Result<()> {
        self.execute(DELETE, [id].as_ref()).await?;
        Ok(())
    }
}
