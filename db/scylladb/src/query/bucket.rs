use anyhow::Result;
use scylla::{serialize::value::SerializeCql, transport::session::TypedRowIter, CachingSession};
use uuid::Uuid;

use crate::{db::ScyllaDb, model::bucket::BucketModel};

pub const INSERT: &str = "INSERT INTO \"hyperbase\".\"buckets\" (\"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"path\", \"opt_ttl\") VALUES (?, ?, ?, ?, ?, ?, ?)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"path\", \"opt_ttl\" FROM \"hyperbase\".\"buckets\" WHERE \"id\" = ?";
pub const SELECT_MANY_BY_PROJECT_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"path\", \"opt_ttl\" FROM \"hyperbase\".\"buckets\" WHERE \"project_id\" = ?";
pub const SELECT_ALL: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"path\", \"opt_ttl\" FROM \"hyperbase\".\"buckets\"";
pub const UPDATE: &str = "UPDATE \"hyperbase\".\"buckets\" SET \"updated_at\" = ?, \"name\" = ?, \"opt_ttl\" = ? WHERE \"id\" = ?";
pub const DELETE: &str = "DELETE FROM \"hyperbase\".\"buckets\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession) {
    hb_log::info(Some("🔧"), "[ScyllaDB] Setting up buckets table");

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"buckets\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"project_id\" uuid, \"name\" text, \"path\" text, \"opt_ttl\" bigint, PRIMARY KEY (\"id\"))", &[]).await.unwrap();
    cached_session
        .get_session()
        .query(
            "CREATE INDEX IF NOT EXISTS ON \"hyperbase\".\"buckets\" (\"project_id\")",
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

    pub async fn select_many_buckets_after_id_with_limit(
        &self,
        after_id: &Option<Uuid>,
        limit: &i32,
    ) -> Result<TypedRowIter<BucketModel>> {
        let mut query = SELECT_ALL.to_owned();
        let mut values: Vec<Box<dyn SerializeCql>> = Vec::with_capacity(2);

        if let Some(after_id) = after_id {
            values.push(Box::new(after_id));
            query += " WHERE \"id\" > ?";
        }

        values.push(Box::new(limit));
        query += " ORDER BY \"id\" ASC LIMIT ?";

        Ok(self.execute(&query, values).await?.rows_typed()?)
    }

    pub async fn update_bucket(&self, value: &BucketModel) -> Result<()> {
        self.execute(
            UPDATE,
            &(
                value.updated_at(),
                value.name(),
                value.opt_ttl(),
                value.id(),
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_bucket(&self, id: &Uuid) -> Result<()> {
        self.execute(DELETE, [id].as_ref()).await?;
        Ok(())
    }
}
