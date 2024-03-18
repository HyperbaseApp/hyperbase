use anyhow::Result;
use scylla::{transport::session::TypedRowIter, CachingSession};
use uuid::Uuid;

use crate::{db::ScyllaDb, model::bucket::BucketModel};

pub const INSERT: &str = "INSERT INTO \"hyperbase\".\"buckets\" (\"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"path\") VALUES (?, ?, ?, ?, ?, ?)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\" FROM \"hyperbase\".\"buckets\", \"path\" WHERE \"id\" = ?";
pub const SELECT_MANY_BY_PROJECT_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"path\" FROM \"hyperbase\".\"buckets\" WHERE \"project_id\" = ?";
pub const UPDATE: &str = "UPDATE \"hyperbase\".\"buckets\" SET \"updated_at\" = ?, \"name\" = ? WHERE \"id\" = ?";
pub const DELETE: &str = "DELETE FROM \"hyperbase\".\"buckets\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession) {
    hb_log::info(Some("🔧"), "ScyllaDB: Setting up buckets table");

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"buckets\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"project_id\" blob, \"name\" text, \"path\" text, PRIMARY KEY (\"id\"))", &[]).await.unwrap();

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
    pub async fn insert_bucket(&self, value: &BucketModel) -> Result<()> {
        self.execute(INSERT, value).await?;
        Ok(())
    }

    pub async fn select_bucket(&self, id: &Uuid) -> Result<BucketModel> {
        Ok(self
            .execute(SELECT, [id].as_ref())
            .await?
            .first_row_typed()?)
    }

    pub async fn select_many_buckets_by_project_id(
        &self,
        project_id: &Uuid,
    ) -> Result<TypedRowIter<BucketModel>> {
        Ok(self
            .execute(SELECT_MANY_BY_PROJECT_ID, [project_id].as_ref())
            .await?
            .rows_typed()?)
    }

    pub async fn update_bucket(&self, value: &BucketModel) -> Result<()> {
        self.execute(UPDATE, &(value.updated_at(), value.name(), value.id()))
            .await?;
        Ok(())
    }

    pub async fn delete_bucket(&self, id: &Uuid) -> Result<()> {
        self.execute(DELETE, [id].as_ref()).await?;
        Ok(())
    }
}
